#include "vsomeipc.h"
#include "application.h"


#include <cassert>
#include <iostream>
#include <optional>
#include <utility>
#include <thread>

application_t create_application(const char* name) {
    auto af = application::create(name);
    if (af) {
        return new std::shared_ptr<application>(af);
    }
    return nullptr;
}

void application_delete(application_t app) {
    if (app && *app) {
        delete app;
    }
}

char const* application_get_name(application_t app) {
    assert(app && *app);
    return (*app)->name().c_str();
}

struct message_header make_message_header(std::shared_ptr<vsomeip::message> const& msg) {
    struct message_header hdr {
            .service = msg->get_service(),
            .instance = msg->get_instance(),
            .method = msg->get_method(),
            .client = msg->get_client(),
            .session = msg->get_session(),
            .proto_version = msg->get_protocol_version(),
            .if_version = msg->get_interface_version(),
            .message_type = (message_type) msg->get_message_type(),
            .return_code = (return_code) msg->get_return_code(),
            .is_initial = msg->is_initial(),
            .is_reliable = msg->is_reliable(),
            .data = msg->get_payload() ? msg->get_payload()->get_data() : nullptr,
            .data_size = msg->get_length(),
    };
    return hdr;
}

void application_register_handlers(
        application_t app,
        state_handler_t state_handler,
        message_handler_t msg_handler,
        void const* object)
{
    assert(app && *app);
    if (state_handler) {
        (*app)->setup_state_handler(
            [state_handler, object](state_type_ce state) { state_handler(state, object); }
        );
    }
    if (msg_handler) {
        (*app)->setup_msg_handler(
                [msg_handler, object](std::shared_ptr<vsomeip::message> const& msg) {
                    msg_handler(
                        make_message_header(msg),
                        new std::shared_ptr<vsomeip::payload>(msg->get_payload()),
                        object );
        });
    }
}

payload_t application_payload_create(application_t app, uint8_t const* data, uint32_t size) {
    assert(app && *app);
    auto pl = (*app)->create_payload(data, size);
    if (pl)
        return new std::shared_ptr<vsomeip::payload>(pl);
    return nullptr;
}

payload_t payload_create_empty(application_t app) {
    assert(app && *app);
    auto pl = (*app)->create_payload_empty();
    if (pl)
        return new std::shared_ptr<vsomeip::payload>(pl);
    return nullptr;
}

void payload_destroy(payload_t pl) {
    delete pl;
}

static vsomeip::message_type_e from(message_type mt) {
    switch(mt) {
        case MT_REQUEST: return vsomeip::message_type_e::MT_REQUEST;
        case MT_REQUEST_NO_RETURN: return vsomeip::message_type_e::MT_REQUEST_NO_RETURN;
        case MT_NOTIFICATION: return vsomeip::message_type_e::MT_NOTIFICATION;
        case MT_REQUEST_ACK: return vsomeip::message_type_e::MT_REQUEST_ACK;
        case MT_REQUEST_NO_RETURN_ACK: return vsomeip::message_type_e::MT_REQUEST_NO_RETURN_ACK;
        case MT_NOTIFICATION_ACK: return vsomeip::message_type_e::MT_NOTIFICATION_ACK;
        case MT_RESPONSE: return vsomeip::message_type_e::MT_RESPONSE;
        case MT_ERROR: return vsomeip::message_type_e::MT_ERROR;
        case MT_RESPONSE_ACK: return vsomeip::message_type_e::MT_RESPONSE_ACK;
        case MT_ERROR_ACK: return vsomeip::message_type_e::MT_ERROR_ACK;
        case MT_UNKNOWN: return vsomeip::message_type_e::MT_UNKNOWN;
        default: {
            std::cerr << "Invalid message_type from Rust-FFI: 0x" << std::hex << (int)mt << "\n";
            exit(1);
        }
    }
}

static vsomeip::return_code_e from(return_code rt) {
    switch(rt) {
        case E_OK: return vsomeip::return_code_e::E_OK;
        case E_NOT_OK: return vsomeip::return_code_e::E_NOT_OK;
        case E_UNKNOWN_SERVICE: return vsomeip::return_code_e::E_UNKNOWN_SERVICE;
        case E_UNKNOWN_METHOD: return vsomeip::return_code_e::E_UNKNOWN_METHOD;
        case E_NOT_READY: return vsomeip::return_code_e::E_NOT_READY;
        case E_NOT_REACHABLE: return vsomeip::return_code_e::E_NOT_REACHABLE;
        case E_TIMEOUT: return vsomeip::return_code_e::E_TIMEOUT;
        case E_WRONG_PROTOCOL_VERSION: return vsomeip::return_code_e::E_WRONG_PROTOCOL_VERSION;
        case E_WRONG_INTERFACE_VERSION: return vsomeip::return_code_e::E_WRONG_INTERFACE_VERSION;
        case E_MALFORMED_MESSAGE: return vsomeip::return_code_e::E_MALFORMED_MESSAGE;
        case E_WRONG_MESSAGE_TYPE: return vsomeip::return_code_e::E_WRONG_MESSAGE_TYPE;
        case E_UNKNOWN: return vsomeip::return_code_e::E_UNKNOWN;
        default: {
            std::cerr << "Invalid return_code from Rust-FFI: 0x" << std::hex << (int)rt << "\n";
            exit(1);
        }
    }
}

message_t application_create_message(application_t app,
                                     service_id service,
                                     instance_id instance,
                                     method_id  method,
                                     session_id session,
                                     message_type message_type,
                                     return_code return_code,
                                     uint8_t const* data,
                                     uint32_t data_size) {
    assert(app && *app);
    auto msg = (*app)->create_message();
    if (msg) {
        msg->set_service(service);
        msg->set_instance(instance);
        msg->set_method(method);
        msg->set_session(session);
        msg->set_message_type(from(message_type));
        msg->set_return_code(from(return_code));
        if (data_size > 0) {
            assert(data != nullptr);
            msg->set_payload((*app)->create_payload(data, data_size));
        }
        return new std::shared_ptr<vsomeip::message>(msg);
    }
    return nullptr;
}

void message_destroy(message_t msg) {
    delete msg;
}

void application_request_service(application_t app,
                                 service_id service,
                                 instance_id instance,
                                 major_version major,
                                 minor_version minor,
                                 availability_handler_t avail_handler,
                                 void const* object)
{
    assert(app && *app);
    (*app)->setup_avail_handler(service, instance, major,
        [avail_handler, object](vsomeip::service_t svc, vsomeip::instance_t inst, bool avail) {
            avail_handler(svc, inst, avail ? AS_AVAILABLE : AS_UNAVAILABLE, object);}
    );
    (*app)->request_service(service, instance, major, minor);
}

void application_release_service(application_t app, service_id service, instance_id instance, major_version major) {
    assert(app && *app);
    (*app)->clear_avail_handler(service, instance, major);
    (*app)->release_service(service, instance);
}

void application_offer_service(application_t app, service_id service, instance_id instance,
                               major_version major, minor_version  minor)
{
    assert(app && *app);
    (*app)->offer_service(service, instance, major, minor);
}

void application_stop_offer_service(application_t app, service_id  service, instance_id instance,
                                    major_version major, minor_version minor)
{
    assert(app && *app);;
    (*app)->stop_offer_service(service, instance, major, minor);
}

void application_offer_event(application_t app, service_id service, instance_id instance, notifier_id notifier,
                             eventgroup_id const* event_groups, uint32_t event_groups_size, bool is_field,
                             uint32_t cycle, bool change_resets_cycle, bool update_on_change)
{
    assert(app && *app);
    assert(event_groups != nullptr);
    std::set<vsomeip::eventgroup_t> event_groups_set{};
    for(int i = 0; i < event_groups_size; ++i) {
        event_groups_set.emplace(event_groups[i]);
    }
    (*app)->offer_event(service, instance, notifier, event_groups_set,
                        is_field ? vsomeip::event_type_e::ET_FIELD : vsomeip::event_type_e::ET_EVENT,
                        std::chrono::milliseconds(cycle),change_resets_cycle, update_on_change);
}

void application_stop_offer_event(application_t app, service_id service, instance_id instance, notifier_id notifier)
{
    assert(app && *app);
    (*app)->stop_offer_event(service, instance, notifier);
}

void application_request_event(application_t app, service_id service, instance_id instance, notifier_id notifier,
                               eventgroup_id const* event_groups, uint32_t event_groups_size, bool is_field)
{
    assert(app && *app);
    assert(event_groups != nullptr);
    std::set<vsomeip::eventgroup_t> event_groups_set{};
    for(int i = 0; i < event_groups_size; ++i) {
        event_groups_set.emplace(event_groups[i]);
    }
    (*app)->request_event(service, instance, notifier, event_groups_set,
                          is_field ? vsomeip::event_type_e::ET_FIELD : vsomeip::event_type_e::ET_EVENT);
}

void application_release_event(application_t app, service_id service, instance_id instance, notifier_id notifier)
{
    assert(app && *app);
    (*app)->release_event(service, instance, notifier);
}

void application_subscribe_event(application_t app, service_id service, instance_id instance, eventgroup_id eg,
                                 notifier_id event, major_version version)
{
    assert(app && *app);
    (*app)->subscribe(service, instance, eg, version, event);
}

void application_unsubscribe_event(application_t app, service_id service, instance_id instance, eventgroup_id eg)
{
    assert(app && *app);
    (*app)->unsubscribe(service, instance, eg);
}

void application_notify(application_t app, service_id service, instance_id instance, notifier_id notifier,
                                   bool force_send, uint8_t const* data, uint32_t data_len)
{
    assert(app && *app);
    (*app)->notify(service, instance, notifier, force_send, data, data_len);
}

session_id application_send_request(application_t app, service_id service, instance_id instance, method_id method,
                              major_version major, bool reliable, uint8_t const* data, uint32_t data_len)
{
    assert(app && *app);
    return (*app)->send_request(service, instance, method, major, data, data_len, reliable);
}

void application_send_response(application_t app, service_id service, instance_id instance, method_id method,
                               client_id client, session_id session, major_version major, bool reliable,
                               enum return_code rc, uint8_t const* data, uint32_t data_len)
{
    assert(app && *app);
    (*app)->send_response(service, instance, method, client, session, major, reliable,from(rc), data, data_len);
}

void application_send_error(application_t app, service_id service, instance_id instance, method_id method,
                            client_id client, session_id session, major_version major, bool reliable, enum return_code rc)
{
    assert(app && *app);
    (*app)->send_error(service, instance, method, client, session, major, reliable, from(rc));
}

PayloadInfo payload_get_info(payload_t pl) {
    assert(pl);
    if (*pl){
        return PayloadInfo{ (*pl)->get_data() , static_cast<uint32_t>((*pl)->get_length())};
    } else {
        return PayloadInfo{ nullptr, 0};
    }
}
