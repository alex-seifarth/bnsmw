mod scenario1;

#[tokio::main]
async fn main() {
    // let (jhp, jhc) = scenario1::start_scenario1().await;
    

    // let mut app = VSomeipApplication::create("app1").expect("Cannot create app1");
    // 
    // let (sender, mut recv) = tokio::sync::mpsc::unbounded_channel();
    // app.setup_channel_callbacks(sender);
    // 
    // let mut interval = time::interval(Duration::from_secs(2));
    // let mut counter = 0u32;
    // let mut regist_status = false;

    // loop {
    //     tokio::select! {
    //         _ = tokio::signal::ctrl_c() => break,
    //     }
    // }
    // let _ = jhp.await;
    // let _ = jhc.await;
    // return;

    // println!("Running event loop\n");
    // loop {
    //     tokio::select! {
    //         _ = tokio::signal::ctrl_c() => break,
    //         msg = recv.recv() => {
    //             println!("vsomeip command received: {:?}", msg);
    //             if msg.is_some() {
    //                 match msg.unwrap() {
    //                     VSomeipMessage::RegistrationState(registered) => {
    //                         regist_status = registered;
    //                         if registered {
    //                             setup_consumer(&app)
    //                         }
    //                     }
    //                     VSomeipMessage::ServiceAvailability{ service_id: _, instance_id: _, avail} => {
    //                         if avail {
    //                             println!("subscribing");
    //                             app.subscribe(ServiceID::from(0x1234), InstanceID::from(1), EventGroupID::from(2),MethodID::from(0x8001));
    //                         }
    //                     }
    //                     VSomeipMessage::Message(_) => {}
    //                 }
    //             }
    //         },
    //         _ = interval.tick() => {
    //             if regist_status {
    //                 match counter & 0x7 {
    //                     0 => {
    //                         app.offer_service(ServiceID::from(0x1234), InstanceID::from(1), InterfaceVersion::make_version(2, 1));
    //                         app.offer_event_seg(ServiceID::from(0x1234), InstanceID::from(1), MethodID::from(0x8001),
    //                             EventGroupID::from(2), true, None, false, true);
    //                         println!("0x8001: 0");
    //                         app.notify(ServiceID::from(0x1234), InstanceID::from(1), MethodID::from(0x8001),
    //                             &Bytes::from("0"), false);
    //                     },
    //                     1 => {
    //                         println!("0x8001: 1");
    //                         app.notify(ServiceID::from(0x1234), InstanceID::from(1), MethodID::from(0x8001),
    //                             &Bytes::from("1"), false);
    //                     },
    //                     2 => {
    //                         println!("0x8001: 2");
    //                         app.notify(ServiceID::from(0x1234), InstanceID::from(1), MethodID::from(0x8001),
    //                             &Bytes::from("2"), false);
    //                     },
    //                     3 => {
    //                         println!("0x8001: 3");
    //                         app.notify(ServiceID::from(0x1234), InstanceID::from(1), MethodID::from(0x8001),
    //                             &Bytes::from("3"), false);
    //                     },
    //                     4 => {
    //                         println!("0x8001: 4");
    //                         app.notify(ServiceID::from(0x1234), InstanceID::from(1), MethodID::from(0x8001),
    //                             &Bytes::from("4"), false);
    //                     },
    //                     5 => {
    //                         println!("0x8001: 5");
    //                         app.notify(ServiceID::from(0x1234), InstanceID::from(1), MethodID::from(0x8001),
    //                             &Bytes::from("5"), false);
    //                     },
    //                     6 => {
    //                         println!("0x8001: 6");
    //                         app.notify(ServiceID::from(0x1234), InstanceID::from(1), MethodID::from(0x8001),
    //                             &Bytes::from("6"), false);
    //                     },
    //                     7 => {
    //                         app.stop_offer_event(ServiceID::from(0x1234), InstanceID::from(1), MethodID::from(0x8001));
    //                         app.stop_offer_service(ServiceID::from(0x1234), InstanceID::from(1), InterfaceVersion::make_version(2, 1));
    //                     },
    //                     _ => {}
    //                 };
    //                 counter += 1;
    //             }
    //         }
    //     }
    // }
    // release_consumer(&app);
    // println!("Done");
}
// 
// fn setup_consumer(app: &VSomeipApplication) {
//     app.request_event_sge(ServiceID::from(0x1234), InstanceID::from(1), MethodID::from(0x8001),
//                           EventGroupID::from(2), true);
//     app.request_service( ServiceID::from(0x1234), InstanceID::from(1), InterfaceVersion::make_major(2));
// }
//  
// fn release_consumer(app: &VSomeipApplication) {
//     app.release_service( ServiceID::from(0x1234), InstanceID::from(1), InterfaceVersion::make_major(2))
// }