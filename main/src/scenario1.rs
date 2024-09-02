// use std::time::Duration;
// use bytes::Bytes;
// use tokio::task::JoinHandle;
// use tokio::time;
// use vsomeiprs::{InstanceID, InterfaceVersion, MajorVersion, MessageType, MethodID, ReturnCode, ServiceID, VSomeipMessage};
//
// static SERVICE_ID: ServiceID = ServiceID(0x7644);
// static INSTANCE_ID: InstanceID = InstanceID(1);
// static METHOD_ID: MethodID = MethodID(42);
// static MAJOR: u8 = 2;
// static MINOR: u32 = 1;
//
// pub async fn start_scenario1() -> (JoinHandle<()>, JoinHandle<()>) {
//     ( tokio::spawn(provider()), tokio::spawn(consumer()) )
// }
//
// async fn provider() {
//     let mut app = vsomeiprs::VSomeipApplication::create("sc1p").expect("Cannot create sc1p");
//     let (sender, mut recv) = tokio::sync::mpsc::unbounded_channel();
//     app.setup_channel_callbacks(sender);
//
//     loop {
//         tokio::select! {
//             _ = tokio::signal::ctrl_c() => {
//                 app.stop_offer_service(SERVICE_ID, INSTANCE_ID, InterfaceVersion::make_version(MAJOR, MINOR));
//                 break
//             },
//             msgo = recv.recv() => {
//                 if let Some(msg) = msgo {
//                     match msg {
//                         VSomeipMessage::RegistrationState(avail) => {
//                             if avail {
//                                 app.offer_service(SERVICE_ID,
//                                     INSTANCE_ID,
//                                     InterfaceVersion::make_version(MAJOR, MINOR))
//                             }
//                         },
//                         VSomeipMessage::ServiceAvailability{ .. } => {},
//                         VSomeipMessage::Message(vmsg) => {
//                             println!("S1 Provider got: {}", vmsg);
//                             match vmsg {
//                                 MessageType::Request{header, data} => {
//                                     app.send_response(&header, ReturnCode::Ok, data.as_bytes_ref());
//                                 },
//                                 MessageType::RequestNoReturn{ .. } => {},
//                                 MessageType::Response{ .. } => {},
//                                 MessageType::Error{ .. } => {},
//                                 MessageType::Notification{ .. } => {}
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
//
// async fn consumer() {
//     let mut counter = 0u32;
//     let mut svc_available = false;
//     let mut interval = time::interval(Duration::from_millis(100));
//     let mut app = vsomeiprs::VSomeipApplication::create("sc1c").expect("Cannot create sc1c");
//     let (sender, mut recv) = tokio::sync::mpsc::unbounded_channel();
//     app.setup_channel_callbacks(sender);
//
//     loop {
//         tokio::select! {
//             _ = tokio::signal::ctrl_c() => {
//                 app.release_service(SERVICE_ID, INSTANCE_ID, InterfaceVersion::make_version(MAJOR, MINOR));
//                 tokio::time::sleep(Duration::from_millis(250)).await;
//                 break
//             },
//             msgo = recv.recv() => {
//                  if let Some(msg) = msgo {
//                     match msg {
//                         VSomeipMessage::RegistrationState(avail) => {
//                             if avail {
//                                app.request_service(SERVICE_ID, INSTANCE_ID, InterfaceVersion::make_version(MAJOR, MINOR));
//                             }
//                         },
//                         VSomeipMessage::ServiceAvailability{ service_id, instance_id, avail } => {
//                             svc_available = avail;
//                             println!("Availability: {:04x}.{:04x}: {}", service_id, instance_id, avail);
//                         },
//                         VSomeipMessage::Message(vmsg) => {
//                             println!("S1 Consumer got: {}", vmsg);
//                             // match vmsg {
//                             //     MessageType::Request{header, data} => {},
//                             //     MessageType::RequestNoReturn{ .. } => {},
//                             //     MessageType::Response{ .. } => {},
//                             //     MessageType::Error{ .. } => {},
//                             //     MessageType::Notification{ .. } => {}
//                             // }
//                         }
//                     }
//                 }
//             }
//             _ = interval.tick() => {
//                 if svc_available {
//                     match counter % 3 {
//                         0 => {
//                             let _ = app.send_request(SERVICE_ID,
//                                                      INSTANCE_ID,
//                                                      METHOD_ID,
//                                                      MajorVersion(MAJOR),
//                                                      &Bytes::from("101"), false);
//                         },
//                         1 => {},
//                         2 => {},
//                         _ => {},
//                     }
//                     counter += 1;
//                 } else {
//                     counter = 0;
//                 }
//             }
//         }
//     }
// }