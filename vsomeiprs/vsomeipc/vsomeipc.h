// SPDX-License-Identifier: MPL-2.0
//
// Copyright (C) 2024 Alexander Seifarth
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#ifndef VSOMEIPC_H_
#define VSOMEIPC_H_

enum state_type_ce {
    DEREGISTERED = 0,
    REGISTERED = 1,
};

enum availability_state_e {
    AS_UNAVAILABLE = 0,
    AS_AVAILABLE = 1,
};

enum message_type {
    MT_REQUEST = 0x00,
    MT_REQUEST_NO_RETURN = 0x01,
    MT_NOTIFICATION = 0x02,
    MT_REQUEST_ACK = 0x40,
    MT_REQUEST_NO_RETURN_ACK = 0x41,
    MT_NOTIFICATION_ACK = 0x42,
    MT_RESPONSE = 0x80,
    MT_ERROR = 0x81,
    MT_RESPONSE_ACK = 0xC0,
    MT_ERROR_ACK = 0xC1,
    MT_UNKNOWN = 0xFF
};

enum return_code {
    E_OK = 0x00,
    E_NOT_OK = 0x01,
    E_UNKNOWN_SERVICE = 0x02,
    E_UNKNOWN_METHOD = 0x03,
    E_NOT_READY = 0x04,
    E_NOT_REACHABLE = 0x05,
    E_TIMEOUT = 0x06,
    E_WRONG_PROTOCOL_VERSION = 0x07,
    E_WRONG_INTERFACE_VERSION = 0x08,
    E_MALFORMED_MESSAGE = 0x09,
    E_WRONG_MESSAGE_TYPE = 0x0A,
    E_UNKNOWN = 0xFF
};

#ifdef CXX_BUILD

#include <vsomeip/vsomeip.hpp>
#include <cstdint>
#include <memory>

class application;
using application_t = std::shared_ptr<application>*;

using message_t = std::shared_ptr<vsomeip::message>*;
using payload_t = std::shared_ptr<vsomeip::payload>*;

using service_id = vsomeip::service_t;
using instance_id = vsomeip::instance_t;
using method_id = vsomeip::method_t;
using notifier_id = vsomeip::event_t;
using client_id = vsomeip::client_t;
using session_id = vsomeip::session_t;
using eventgroup_id = vsomeip::eventgroup_t;
using protocol_version = vsomeip::protocol_version_t;
using interface_version = vsomeip::interface_version_t;
using major_version = vsomeip::major_version_t;
using minor_version = vsomeip::minor_version_t;

#else

#include <stdint.h>
#include <stdbool.h>

typedef void* message_t;
typedef void* payload_t;
typedef void* application_t;
typedef uint16_t service_id;
typedef uint16_t instance_id;
typedef uint16_t method_id;
typedef uint16_t notifier_id;
typedef uint16_t client_id;
typedef uint16_t session_id;
typedef uint16_t eventgroup_id;
typedef uint8_t protocol_version;
typedef uint8_t interface_version;
typedef uint8_t major_version;
typedef uint32_t minor_version;

#endif

#ifdef __cplusplus
extern "C" {
#endif

    typedef void (*state_handler_t)(enum state_type_ce state, void const* target);
    typedef void (*availability_handler_t)(service_id svc_id, instance_id inst_id, enum availability_state_e avail, void const* target);

    struct message_header {
        service_id service;
        instance_id instance;
        method_id  method;
        client_id client;
        session_id session;
        protocol_version proto_version;
        interface_version if_version;
        enum message_type message_type;
        enum return_code return_code;
        bool is_initial;
        bool is_reliable;
        uint8_t const* data;
        uint32_t data_size;
    };

    typedef void (*message_handler_t)(struct message_header header, payload_t payload, void const* target);

    // application handling
    application_t create_application(const char* name);
    void application_register_handlers(application_t app,
                                       state_handler_t state_handler,
                                       message_handler_t msg_handler,
                                       void const* object);
    void application_delete(application_t app);
    char const* application_get_name(application_t app);

    session_id send_request(application_t app, uint8_t const* data, uint32_t data_len);


    void application_request_service(application_t app, service_id service, instance_id instance,
                                     major_version major, minor_version minor,
                                     availability_handler_t avail_handler, void const* object);
    void application_release_service(application_t app, service_id service, instance_id instance, major_version major);
    void application_offer_service(application_t app, service_id service, instance_id instance,
                                   major_version major, minor_version  minor);
    void application_stop_offer_service(application_t app, service_id  service, instance_id instance,
                                        major_version major, minor_version minor);
    void application_offer_event(application_t app, service_id service, instance_id instance, notifier_id notifier,
            eventgroup_id const* event_groups, uint32_t event_groups_size, bool is_field,
            uint32_t cycle, bool change_resets_cycle, bool update_on_change);
    void application_stop_offer_event(application_t app, service_id service, instance_id instance, notifier_id notifier);
    void application_request_event(application_t app, service_id service, instance_id instance, notifier_id notifier,
                                   eventgroup_id const* event_groups, uint32_t event_groups_size, bool is_field);
    void application_release_event(application_t app, service_id service, instance_id instance, notifier_id notifier);
    void application_subscribe_event(application_t app, service_id service, instance_id instance, eventgroup_id eg,
                                     notifier_id event, major_version version);
    void application_unsubscribe_event(application_t app, service_id service, instance_id instance, eventgroup_id eg);

    //    void subscribe_with_debounce(vsomeip::service_t service, vsomeip::instance_t instance,
    //                                 vsomeip::eventgroup_t event_group, vsomeip::major_version_t major,
    //                                 vsomeip::event_t event, vsomeip::debounce_filter_t const& filter);

    void application_notify(application_t app, service_id service, instance_id instance, notifier_id notifier,
                            bool force_send, uint8_t const* data, uint32_t data_len);
    session_id application_send_request(application_t app, service_id service, instance_id instance, method_id method,
                            major_version major, bool reliable, uint8_t const* data, uint32_t data_len);
    void application_send_response(application_t app, service_id service, instance_id instance, method_id method,
                                   client_id client, session_id session, major_version major, bool reliable,
                                   enum return_code rc, uint8_t const* data, uint32_t data_len);
    void application_send_error(application_t app, service_id service, instance_id instance, method_id method,
                                client_id client, session_id session, major_version major, bool reliable,  enum return_code rc);


// payload handling
    struct PayloadInfo {
        uint8_t* data;
        uint32_t len;
    };

    payload_t application_payload_create(application_t app, uint8_t const* data, uint32_t size);
    payload_t payload_create_empty(application_t app);
    void payload_destroy(payload_t pl);
    struct PayloadInfo payload_get_info(payload_t pl);

    // message handling
    message_t application_create_message(application_t app,
                                         service_id service,
                                         instance_id instance,
                                         method_id  method,
                                         session_id session,
                                         enum message_type message_type,
                                         enum return_code return_code,
                                         uint8_t const* data,
                                         uint32_t data_size);
    void application_send_msg(application_t app, message_t msg);
    void message_destroy(message_t msg);


#ifdef __cplusplus
}
#endif

#endif  /*VSOMEIPC_H_*/
