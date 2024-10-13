// SPDX-License-Identifier: MPL-2.0
//
// Copyright (C) 2024 Alexander Seifarth
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod types;
pub use types::*;

use std::ffi::{c_char, CString};
use std::fmt::{Debug, Formatter};
use std::time::Duration;
use bytes::Bytes;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Debug)]
pub enum VSomeipMessage {
    RegistrationState(bool),
    ServiceAvailability{ service_id: u16, instance_id: u16, avail: bool },
    Message(MessageType)
}

/// Wraps a vsomeip::application object from the C++ world.
/// Via `create()` and `drop()` this implements a RAII pattern - the creation immediately
/// creates the vsomeip application object, initializes and starts it. At 'destruction' (i.e.
/// `drop()`) the application is stopped (joins the I/O threads), removes it from the runtime and
/// finally deletes the application object. Most of this is implemented in the C++ part of the
/// library.
pub struct VSomeipApplication {
    app: ffi::application_t,
    sender2: Box<UnboundedSender<VSomeipMessage>>,
}

impl Drop for VSomeipApplication {
    fn drop(&mut self) {
        unsafe { ffi::application_delete(self.app) }
    }
}

unsafe impl Send for VSomeipApplication {}

unsafe impl Sync for VSomeipApplication {}

impl VSomeipApplication {
    /// Creates a new vsomeip application object.
    /// - starts the vsomeip application including its i/o threads,
    /// - creates async channels (tokio::sync::mpsc::Unbounded...) and registers the callback handlers
    /// - returns the application and the channel receiver
    ///
    /// # Args
    /// - name    The name of the application object. Note that vsomeip might modify it if not
    ///           unique.
    ///
    /// # Returns
    /// The application object and the channel receiver are returned in case of success (OK).
    pub fn create(name: &str) -> Result<(Self, UnboundedReceiver<VSomeipMessage>), ()> {
        let name_cstr = CString::new(name).unwrap();
        let name_c: *const c_char = name_cstr.as_ptr() as *const c_char;
        let app = unsafe { ffi::create_application(name_c) };
        if app.is_null() {
            return Err(());
        }
        let (sender, recv) = tokio::sync::mpsc::unbounded_channel();
        let mut application = VSomeipApplication {app, sender2: Box::new(sender)};
        application.setup_channel_callbacks();
        Ok( (application, recv) )
    }

    /// Registers the vsomeip callbacks (state, availability, message).
    /// Each callback invocation is transformed into a `VSomeipMessage` and sent in the unbounded
    /// channel.
    /// This method must be invoked only once!
    fn setup_channel_callbacks(&mut self) {
        // TODO panic when this method is called more than once.
        unsafe {
            let sender_ptr = &(*self.sender2) as *const UnboundedSender<VSomeipMessage>;
            ffi::application_register_handlers(
                self.app,
                Some(state_handler),
                Some(message_handler2),
                sender_ptr as *const std::os::raw::c_void);
        }
    }

    /// Requests a SOME/IP service.
    /// A consumer must request a desired service before it can use it. Once it is requested the
    /// service's availability notifications will be sent to the application.
    pub fn request_service(&self, service_id: ServiceID, instance_id: InstanceID, version: InterfaceVersion)
    {
        unsafe {
            let sender_ptr = &(*self.sender2) as *const UnboundedSender<VSomeipMessage>;
            ffi::application_request_service(self.app, service_id.id(), instance_id.id(),
                                             version.major.id(), version.minor.id(),
                                             Some(avail_handler),
                                             sender_ptr as *const std::os::raw::c_void);
        }
    }

    /// Releases a requested SOME/IP service.
    pub fn release_service(&self, service_id: ServiceID, instance_id: InstanceID, version: InterfaceVersion) {
        unsafe {
            ffi::application_release_service(self.app, service_id.id(), instance_id.id(), version.major.id());
        }
    }

    /// A provider of a service indicates it's readiness to process requests for the service instance.
    /// NOTE: In a SOME/IP network only one provider can offer a service instance. Nevertheless, it 
    ///      is possible to call this method when there is already a provider for the instance. 
    ///      VSOMEIP will then consider the second and later providers as hot-standby for the 
    ///      currently active provider. Therefore, there will be error message or any other 
    ///      indication that a provider is not the active one.
    pub fn offer_service(&self, service_id: ServiceID, instance_id: InstanceID, version: InterfaceVersion) {
        unsafe {
            ffi::application_offer_service(self.app, service_id.id(), instance_id.id(), 
                                           version.major.id(), version.minor.id())
        }
    }
    
    /// A provider indicates that it is no longer offering the service instance.
    pub fn stop_offer_service(&self, service_id: ServiceID, instance_id: InstanceID, version: InterfaceVersion) {
        unsafe {
            ffi::application_stop_offer_service(self.app, service_id.id(), instance_id.id(),
                                                version.major.id(), version.minor.id())
        }
    }

    pub fn offer_event(&self,  service_id: ServiceID, instance_id: InstanceID, notifier_id: MethodID,
                        event_groups: Vec<EventGroupID>,
                        is_field: bool,
                        cycle: Option<Duration>,
                        change_resets_cycle: bool,
                        update_on_change: bool)
    {
        unsafe {
            ffi::application_offer_event(self.app, service_id.id(), instance_id.id(), notifier_id.id(),
                                         event_groups.as_ptr() as *const ffi::eventgroup_id,
                                         event_groups.len() as u32,
                                         is_field,
                                         cycle.map(|x| x.as_millis() as u32).unwrap_or(0),
                                         change_resets_cycle, update_on_change)
        }
    }

    pub fn offer_event_seg(&self,  service_id: ServiceID, instance_id: InstanceID, notifier_id: MethodID,
                       event_group: EventGroupID,
                       is_field: bool,
                       cycle: Option<Duration>,
                       change_resets_cycle: bool,
                       update_on_change: bool)
    {
        self.offer_event(service_id, instance_id, notifier_id, vec![event_group], is_field,
                        cycle, change_resets_cycle, update_on_change)
    }

    pub fn stop_offer_event(&self, service_id: ServiceID, instance_id: InstanceID, notifier_id: MethodID)
    {
        unsafe {
            ffi::application_stop_offer_event(self.app, service_id.id(), instance_id.id(), notifier_id.id())
        }
    }

    /// Consumers must request (configure) events from SOME/IP services before they can
    /// subscribe to the notifications of these events.
    /// It is important to configure ALL events defined for an event group even when the consumer
    /// is not interested in them. Otherwise, vsomeip will discard initial event notifications
    /// arriving after the first subscription for the event group. This may result in lost
    /// notifications for other consumer subscribing later.
    pub fn request_event(&self,  service_id: ServiceID, instance_id: InstanceID, notifier_id: MethodID,
                       event_groups: Vec<EventGroupID>,
                       is_field: bool)
    {
        unsafe {
            ffi::application_request_event(self.app, service_id.id(), instance_id.id(), notifier_id.id(),
                   event_groups.as_ptr() as *const ffi::eventgroup_id, event_groups.len() as u32, is_field)
        }
    }

    /// Same as `request_event` but for a signle event group
    pub fn request_event_seg(&self, service_id: ServiceID, instance_id: InstanceID, notifier_id: MethodID,
                             event_group: EventGroupID, is_field: bool)
    {
        self.request_event(service_id, instance_id, notifier_id, vec![event_group], is_field)
    }

    /// Release a previously requested event.
    pub fn release_event(&self, service_id: ServiceID, instance_id: InstanceID, notifier_id: MethodID)
    {
        unsafe {
            ffi::application_release_event(self.app, service_id.id(), instance_id.id(), notifier_id.id())
        }
    }

    /// Subscribes a consumer for event/field notifications.
    /// NOTE 1: The event must be registered before.
    /// NOTE 2: SOME/IP subscriptions are not per event, but per event group. So this method
    ///         indeed subscribe to the event group `event_group_id`. The local vsomeip uses the
    ///         `notifier_id` only to filter which event notifications from the event group will
    ///         be forwarded to the application.
    pub fn subscribe(&self, service_id: ServiceID, instance_id: InstanceID, event_group_id: EventGroupID,
                        notifier_id: MethodID, major_version: MajorVersion)
    {
        unsafe {
            ffi::application_subscribe_event(self.app, service_id.id(), instance_id.id(),
                                             event_group_id.id(), notifier_id.id(), major_version.id())
        }
    }

    /// Unsubscribe a consumer from a previously subscribed event group.
    pub fn unsubscribe(&self, service_id: ServiceID, instance_id: InstanceID, event_group_id: EventGroupID)
    {
        unsafe {
            ffi::application_unsubscribe_event(self.app, service_id.id(), instance_id.id(),
                                               event_group_id.id())
        }
    }

    /// Updates the data for an event or field and sends a notification if changed or forced.
    pub fn notify(&self, service_id: ServiceID, instance_id: InstanceID, notifier_id: MethodID,
                  payload: &Bytes, force_notification: bool)
    {
        unsafe {
            ffi::application_notify(self.app, service_id.id(), instance_id.id(), notifier_id.id(),
                force_notification, payload.as_ptr(), payload.len() as u32)
        }
    }

    /// Sends a request message.
    /// # Return
    /// Returns the assigned session id. The response (or error) from the provider will carry the
    /// same session id which allows to link them to the request.
    pub fn send_request(&self, service_id: ServiceID, instance_id: InstanceID, method_id: MethodID,
        major: MajorVersion, payload: &Bytes, reliable: bool) -> SessionID
    { 
        SessionID::from(
        unsafe {
                ffi::application_send_request(self.app, service_id.id(), instance_id.id(), method_id.id(),
                    major.id(), reliable, payload.as_ptr(), payload.len() as u32)
            }
        )
    }

    /// Sends a response message.
    /// # Argument
    /// - source_request        The message header of the linked request.
    pub fn send_response(&self, source_request: &MessageHeader, return_code: ReturnCode, payload: &Bytes) {
        unsafe {
            ffi::application_send_response(self.app,
                                           source_request.service_id.id(),
                                           source_request.instance_id.id(),
                                           source_request.method_id.id(),
                                           source_request.client_id.id(),
                                           source_request.session_id.id(),
                                           source_request.interface_version.major.id(),
                                           source_request.reliable,
                                           return_code_to_ffi(return_code),
                                           payload.as_ptr(),
                                           payload.len() as u32);
        }
    }

    /// Sends an error message.
    /// # Argument
    /// - source_request        The message header of the linked request.
    pub fn send_error(&self, source_request: &MessageHeader, return_code: ReturnCode) {
        unsafe {
            ffi::application_send_error(self.app,
                                        source_request.service_id.id(),
                                        source_request.instance_id.id(),
                                        source_request.method_id.id(),
                                        source_request.client_id.id(),
                                        source_request.session_id.id(),
                                        source_request.interface_version.major.id(),
                                        source_request.reliable,
                                        return_code_to_ffi(return_code));
        }
    }
}

macro_rules! to_sender {
    ($target:ident) => {
        ($target as *mut UnboundedSender<VSomeipMessage>).as_ref().unwrap()
    };
}

extern "C"
fn state_handler(state: ffi::state_type_ce, target: *const std::os::raw::c_void) {
    unsafe {
        // TODO how to react on failed transmission?
        // -> unwrap() ==> panic
        to_sender!(target).send(
            VSomeipMessage::RegistrationState( state == ffi::state_type_ce_REGISTERED)).unwrap();
    }
}

extern "C"
fn avail_handler(svc_id: u16,
                 inst_id: u16,
                 avail: ffi::availability_state_e,
                 target: *const std::os::raw::c_void)
{
    unsafe {
        // TODO how to react on failed transmission?
        // -> unwrap() ==> panic
        to_sender!(target).send(
    VSomeipMessage::ServiceAvailability { service_id: svc_id, instance_id: inst_id,
                avail : avail == ffi::availability_state_e_AS_AVAILABLE }).unwrap()
    }
}

fn make_header(hdr: &ffi::message_header) -> MessageHeader {
    MessageHeader {
        service_id: ServiceID::from(hdr.service),
        instance_id: InstanceID::from(hdr.instance),
        method_id: MethodID::from(hdr.method),
        client_id: ClientID::from(hdr.client),
        session_id: SessionID::from(hdr.session),
        interface_version: InterfaceVersion::make_major(hdr.if_version),
        reliable: hdr.is_reliable,
    }
}

fn map_return_code(rt: ffi::return_code) -> ReturnCode {
    match rt {
        ffi::return_code_E_OK => ReturnCode::Ok,
        ffi::return_code_E_NOT_OK => ReturnCode::NotOk,
        ffi::return_code_E_UNKNOWN_SERVICE => ReturnCode::UnknownService,
        ffi::return_code_E_UNKNOWN_METHOD => ReturnCode::UnknownMethod,
        ffi::return_code_E_NOT_READY => ReturnCode::NotReady,
        ffi::return_code_E_NOT_REACHABLE => ReturnCode::NotReachable,
        ffi::return_code_E_TIMEOUT => ReturnCode::Timeout,
        ffi::return_code_E_WRONG_PROTOCOL_VERSION => ReturnCode::WrongProtocolVersion,
        ffi::return_code_E_WRONG_INTERFACE_VERSION => ReturnCode::WrongInterfaceVersion,
        ffi::return_code_E_MALFORMED_MESSAGE => ReturnCode::MalformedMessage,
        ffi::return_code_E_WRONG_MESSAGE_TYPE => ReturnCode::WrongMessageType,
        ffi::return_code_E_UNKNOWN => ReturnCode::Unknown,
        val => { panic!("Unknown return code {}", val); }
    }
}

fn return_code_to_ffi(rc: ReturnCode) -> ffi::return_code {
    match rc {
        ReturnCode::Ok => ffi::return_code_E_OK,
        ReturnCode::NotOk => ffi::return_code_E_NOT_OK,
        ReturnCode::UnknownService => ffi::return_code_E_UNKNOWN_SERVICE,
        ReturnCode::UnknownMethod => ffi::return_code_E_UNKNOWN_METHOD,
        ReturnCode::NotReady => ffi::return_code_E_NOT_READY,
        ReturnCode::NotReachable => ffi::return_code_E_NOT_REACHABLE,
        ReturnCode::Timeout => ffi::return_code_E_TIMEOUT,
        ReturnCode::WrongProtocolVersion => ffi::return_code_E_WRONG_PROTOCOL_VERSION,
        ReturnCode::WrongInterfaceVersion => ffi::return_code_E_WRONG_INTERFACE_VERSION,
        ReturnCode::MalformedMessage => ffi::return_code_E_MALFORMED_MESSAGE,
        ReturnCode::WrongMessageType => ffi::return_code_E_WRONG_MESSAGE_TYPE,
        ReturnCode::Unknown => ffi::return_code_E_UNKNOWN,
    }
}


extern "C"
fn message_handler2(
    msg_header: ffi::message_header,
    payload: ffi::payload_t,
    target: *const std::os::raw::c_void)
{
    let data = VSomeipPayload::from(payload);
    let header = make_header(&msg_header);

    let msg = match msg_header.message_type {
        ffi::message_type_MT_REQUEST => MessageType::Request {header, data},
        ffi::message_type_MT_REQUEST_NO_RETURN => MessageType::RequestNoReturn {header, data},
        ffi::message_type_MT_NOTIFICATION => MessageType::Notification {header, data,
            is_initial: msg_header.is_initial},
        ffi::message_type_MT_RESPONSE => MessageType::Response {header, data},
        ffi::message_type_MT_ERROR => MessageType::Error {header, data,
            return_code: map_return_code(msg_header.return_code)},

        // the following vsomeip message types shouldn't be sent upstream from libvsomeip
        // so we ignore them
        ffi::message_type_MT_REQUEST_ACK => { return /* ignored */ },
        ffi::message_type_MT_REQUEST_NO_RETURN_ACK => { return /* ignored */ },
        ffi::message_type_MT_NOTIFICATION_ACK => { return /* ignored */ },
        ffi::message_type_MT_RESPONSE_ACK => { return /* ignored */ },
        ffi::message_type_MT_ERROR_ACK => { return /* ignored */ },
        ffi::message_type_MT_UNKNOWN => { return /* ignored */ },

        // an unknown vsomeip message type usually indicates that vsomeip is in an undefined
        // state, or we have linked to an unsupported vsomeip version.
        val => { panic!("Unknown message type from vsomeip {}", val)}
    };

    unsafe {
        // TODO how to react on failed transmission?
        // -> unwrap() ==> panic
        to_sender!(target).send(VSomeipMessage::Message(msg)).unwrap()
    }
}

/// Encapsulation of a vsomeip::payload object.
pub struct VSomeipPayload {
    payload: ffi::payload_t,
    bytes: Bytes
}

impl Drop for VSomeipPayload {
    fn drop(&mut self) {
        unsafe { ffi::payload_destroy(self.payload) }
    }
}

impl From<ffi::payload_t> for VSomeipPayload {
    fn from(value: ffi::payload_t) -> Self {
        Self{ payload: value, bytes: payload_to_bytes(value) }
    }
}

impl Debug for VSomeipPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.bytes)
    }
}

unsafe impl Send for VSomeipPayload {}

unsafe impl Sync for VSomeipPayload {}

impl VSomeipPayload {

    /// Returns the data within the payload as `Bytes` reference.
    /// NOTE: This involves no copying, but the reference's lifetime is bound to the
    /// VSomeipPayload object.
    pub fn as_bytes_ref(&self) -> &Bytes  {
        &self.bytes
    }
}

fn payload_to_bytes(payload: ffi::payload_t) -> Bytes {
    if payload.is_null() {
        Bytes::new()
    } else {
        unsafe {
            let pli = ffi::payload_get_info(payload);
            if pli.data.is_null() || pli.len == 0 {
                Bytes::new()
            } else {
                Bytes::from_static(std::slice::from_raw_parts(pli.data, pli.len as usize))
            }
        }
    }
}

