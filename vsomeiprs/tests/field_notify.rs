use std::time::Duration;
use bytes::{Buf, BufMut, BytesMut};
use vsomeiprs::{EventGroupID, InstanceID, InterfaceVersion, MajorVersion, MessageType, MethodID, ServiceID, VSomeipApplication, VSomeipMessage};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time;
use tokio::time::timeout;

const SERVICE_ID: ServiceID = ServiceID(0x4711);
const INSTANCE_ID: InstanceID = InstanceID(42);
const NOTIFIER_ID: MethodID = MethodID(0x8002);
const EVENT_GROUP: EventGroupID = EventGroupID(8);
const MAJOR: u8 = 3;
const MINOR: u32 = 28;
const MAX_COUNT_NOTIFICATION:u32 = 50;

/// Test: field-notify
///
/// Creates three vsomeip applications:
/// - routing: setup before the others, acts as routing manager host
/// - provider: Offers a single service interface and one field-event in it, then sends 50 notifications
///             on the field-event.
/// - consumer: Requests the interface of the provider and subscribes to the event. Completes
///             when the final expected event is received.
///
#[tokio::test]
pub async fn main() {
    let (_rtmp, _crecv) = setup_app("routing").await;

    let ph = tokio::spawn(provider());

    match timeout(Duration::from_secs(100), consumer()).await {
        Ok(result) => {
            assert_eq!(result.1, MAX_COUNT_NOTIFICATION);
        }
        Err(_) => panic!("Error - timeout waiting for consumer"),
    }
    let _ = ph.await;
}

async fn provider() {
    let version = InterfaceVersion::make_version(MAJOR, MINOR);
    let mut counter = 0u32;

    // create the provider app before fork ensure that it has the routing manager
    let (papp, mut precv) = setup_app("provider").await;
    papp.offer_event_seg(SERVICE_ID, INSTANCE_ID, NOTIFIER_ID, EVENT_GROUP, true, None, true, true);
    papp.offer_service(SERVICE_ID, INSTANCE_ID, version);

    let mut interval = time::interval(Duration::from_millis(100));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                counter += 1;
                if counter > MAX_COUNT_NOTIFICATION {
                    break;
                } else {
                    let mut pl = BytesMut::with_capacity(4);
                    pl.put_u32(counter);
                    // println!("sending: {}", counter);
                    papp.notify(SERVICE_ID, INSTANCE_ID, NOTIFIER_ID, &pl.freeze(), true);
                }
            },
            _ = precv.recv() => { /*println!("Message {:?}", msg);*/ }
        }
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
    papp.stop_offer_event(SERVICE_ID, INSTANCE_ID, NOTIFIER_ID);
    papp.stop_offer_service(SERVICE_ID, INSTANCE_ID, version);
}

async fn consumer() -> (u32, u32) {
    let version = InterfaceVersion::make_version(MAJOR, MINOR);
    let mut counter: u32;
    let mut notific_counter = 0u32;

    let (capp, mut crecv) = setup_app("consumer").await;
    capp.request_service(SERVICE_ID, INSTANCE_ID, version);
    capp.request_event_seg(SERVICE_ID, INSTANCE_ID, NOTIFIER_ID, EVENT_GROUP, true);
    loop {
        tokio::select! {
            msgo = crecv.recv() => {
                if let Some(msg) = msgo {
                    match msg {
                        VSomeipMessage::RegistrationState(rs) => {
                            if !rs {
                                panic!("Registration lost to vsomeip")
                            }
                        }
                        VSomeipMessage::ServiceAvailability{ service_id, instance_id, avail } => {
                            // println!("Service {:04x}.{:04x} available: {}", service_id, instance_id, avail);
                            if service_id == SERVICE_ID.id() && instance_id == INSTANCE_ID.id() && avail {
                                // println!("Subscribing");
                                capp.subscribe(SERVICE_ID, INSTANCE_ID, EVENT_GROUP, NOTIFIER_ID, MajorVersion(MAJOR));
                            }
                        }
                        VSomeipMessage::Message(m) => {
                            // println!("Received: {}", m);
                            match m {
                                MessageType::Request{ .. } => {}
                                MessageType::RequestNoReturn{ .. } => {}
                                MessageType::Response{ .. } => {}
                                MessageType::Error{ .. } => {}
                                MessageType::Notification{ header, is_initial: _, data } => {
                                    if header.service_id == SERVICE_ID && header.method_id == NOTIFIER_ID {
                                        notific_counter += 1;
                                        let mut datab = data.as_bytes_ref().as_ref();
                                        assert_eq!(datab.len(), 4);
                                        counter = datab.get_u32();
                                        if counter == MAX_COUNT_NOTIFICATION {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    panic!("consumer vsomeip channel closed")
                }
            }
        }
    }
    capp.release_event(SERVICE_ID, INSTANCE_ID, NOTIFIER_ID);
    capp.release_service(SERVICE_ID, INSTANCE_ID, version);
    (notific_counter, counter)
}

async fn setup_app(name: &str) -> (VSomeipApplication, UnboundedReceiver<VSomeipMessage>) {
    let (app, mut recv) = VSomeipApplication::create(name).unwrap();
    loop {
        tokio::select! {
            msg = recv.recv() => {
                match msg {
                    Some(VSomeipMessage::RegistrationState(true)) => {break;},
                    None => { panic!("Channel closed") }
                    _ => {}
                }
            }
        }
    }
    (app, recv)
}
