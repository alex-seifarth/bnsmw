use std::collections::HashMap;
use std::ops::BitXor;
use std::time::Duration;
use bytes::{Buf, BufMut, BytesMut};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time;
use tokio::time::timeout;
use vsomeiprs::{InstanceID, InterfaceVersion, MajorVersion, MessageType, MethodID, ReturnCode, ServiceID, VSomeipApplication, VSomeipMessage};

const SERVICE_ID: ServiceID = ServiceID(0x002a);
const INSTANCE_ID: InstanceID = InstanceID(101);
const METHOD_ID: MethodID = MethodID(0x0002);
const MAJOR: u8 = 2;
const MINOR: u32 = 3;
const MAX_COUNT_REQUESTS:u32 = 50;

/// Test:request-response
///
/// Creates three vsomeip applications:
/// - routing: setup before the others, acts as routing manager host
/// - provider: Offers a single interface and processes requests.
///             Method 1 request returns the input xor-ed with 0x12345678
/// - consumer: Requests the interface of the provider and sends request to the service and
///             checks the reponses
///
#[tokio::test]
pub async fn main() {
    let (_rtmp, _crecv) = setup_app("routing").await;

    let ph = tokio::spawn(provider());

    match timeout(Duration::from_secs(100), consumer()).await {
        Ok(_) => {}
        Err(_) => panic!("Error - timeout waiting for consumer"),
    }
    let _ = ph.await;
}

async fn provider() {
    let version = InterfaceVersion::make_version(MAJOR, MINOR);
    let (papp, mut precv) = setup_app("provider").await;
    papp.offer_service(SERVICE_ID, INSTANCE_ID, version);
    loop {
        tokio::select! {
            msgo = precv.recv() => {
                if let Some(msg) = msgo {
                    match msg {
                        VSomeipMessage::RegistrationState(rs) => { assert!(rs) }
                        VSomeipMessage::ServiceAvailability{ .. } => {}
                        VSomeipMessage::Message(m) => {
                            // println!("P: {}", m);
                            match m {
                                MessageType::Request{ header, data } => {
                                    assert_eq!(header.service_id, SERVICE_ID);
                                    assert_eq!(header.instance_id, INSTANCE_ID);
                                    assert_eq!(header.method_id, METHOD_ID);
                                    assert_eq!(header.interface_version.major.id(), MAJOR);
                                    let mut payload = data.as_bytes_ref().as_ref();
                                    assert_eq!(payload.len(), 4);
                                    let input = payload.get_u32();
                                    let mut resp_pl = BytesMut::with_capacity(4);
                                    resp_pl.put_u32( input.bitxor(0x12345678u32) );
                                    papp.send_response(&header, ReturnCode::Ok, &resp_pl.freeze());

                                    if input == MAX_COUNT_REQUESTS { break }
                                }
                                MessageType::RequestNoReturn{ .. } => { panic!("Unexpected RequestNoReturn") }
                                MessageType::Response{ .. } => { panic!("Unexpected Response") }
                                MessageType::Error{ .. } => { panic!("Unexpected Error") }
                                MessageType::Notification{ .. } => {  panic!("Unexpected Notification") }
                            }
                        }
                    }
                } else {
                    panic!("consumer vsomeip channel closed")
                }
            }
        }
    }
    papp.stop_offer_service(SERVICE_ID, INSTANCE_ID, version);
}

async fn consumer() {
    let version = InterfaceVersion::make_version(MAJOR, MINOR);
    let mut interval = time::interval(Duration::from_millis(100));
    let (capp, mut crecv) = setup_app("consumer").await;
    let mut available = false;
    let mut counter:u32 = 0;
    let mut session_map = HashMap::<u16,u32>::new();
    capp.request_service(SERVICE_ID, INSTANCE_ID, version);
    loop {
        tokio::select!{
            _ = interval.tick() => {
                if available && counter <= MAX_COUNT_REQUESTS {
                   let mut pl = BytesMut::with_capacity(4);
                    pl.put_u32(counter);
                    let session = capp.send_request(SERVICE_ID, INSTANCE_ID, METHOD_ID,
                                                   MajorVersion(MAJOR), &pl.freeze(), false);
                    session_map.insert(session.id(), counter);
                    counter += 1
                }
            }
            msgo = crecv.recv() => {
                if let Some(msg) = msgo {
                    match msg {
                        VSomeipMessage::RegistrationState(rs) => { assert!(rs) }
                        VSomeipMessage::ServiceAvailability{ service_id, instance_id, avail } => {
                            if service_id == SERVICE_ID.id() && instance_id == INSTANCE_ID.id() {
                                available = avail;
                            }
                        }
                        VSomeipMessage::Message(m) => {
                            // println!("C: {}", m);
                            match m {
                                MessageType::Request{ .. } => { panic!("Unexpected Requet") }
                                MessageType::RequestNoReturn{ .. } => { panic!("Unexpected RequestNoReturn") }
                                MessageType::Response{ header, data } => {
                                    assert_eq!(header.service_id, SERVICE_ID);
                                    assert_eq!(header.instance_id, INSTANCE_ID);
                                    assert_eq!(header.method_id, METHOD_ID);
                                    assert_eq!(header.interface_version.major.id(), MAJOR);
                                    let mut payload = data.as_bytes_ref().as_ref();
                                    assert_eq!(payload.len(), 4);
                                    let input = payload.get_u32().bitxor(0x12345678);
                                    assert_eq!(
                                        session_map.get(&header.session_id.id()), Some(&input));
                                    if input >= MAX_COUNT_REQUESTS { break }
                                }
                                MessageType::Error{ .. } => { panic!("Unexpected Error") }
                                MessageType::Notification{ .. } => {  panic!("Unexpected Notification") }
                            }
                        }
                    }
                } else {
                    panic!("consumer vsomeip channel closed")
                }
           }
        }
    }
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
