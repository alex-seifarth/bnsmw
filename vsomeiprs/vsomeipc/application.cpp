#include "application.h"

#include <cassert>
#include <iostream>

std::shared_ptr<application> application::create(std::string const& name) {
    auto runtime = vsomeip::runtime::get();
    assert(runtime);
    auto application= runtime->create_application(name);
    if (!application) {
        std::cerr << "FAILED to create vsomeip::application object [" << name << "]\n";
        return nullptr;
    }
    if (!application->init()) {
        std::cerr << "FAILED to initialize vsomeip::application [" << name << "]\n";
        return nullptr;
    }
    auto af = std::make_shared<::application>(runtime, application);
    af->start();
    return af;
}

application::application(
        std::shared_ptr<vsomeip::runtime> runtime,
        std::shared_ptr<vsomeip::application> application)
        : _runtime{ std::move(runtime) }
        , _application{ std::move(application) }
        , _dispatch_thread{}
        , _state_connected{false}
{}

application::~application() {
    if (_application) {
        _application->clear_all_handler();
        stop();
        _runtime->remove_application(_application->get_name());
        _application.reset();
    }
    _runtime.reset();
}

std::shared_ptr<vsomeip::runtime>& application::runtime() {
    return _runtime;
}

std::string const& application::name() const {
    return _application->get_name();
}

std::shared_ptr<vsomeip::payload> application::create_payload_empty() const {
    return _runtime->create_payload();
}

std::shared_ptr<vsomeip::payload> application::create_payload(uint8_t const* data, uint32_t size) {
    return _runtime->create_payload(data, size);
}

void application::start() {
    assert(!_dispatch_thread.joinable());
    _dispatch_thread = std::thread([this] {
        this->_application->start();
    });
}

void application::stop() {
    _application->stop();
    if (_dispatch_thread.joinable()) {
        _dispatch_thread.join();
    }
}

void application::request_service(vsomeip::service_t service, vsomeip::instance_t instance,
                                  vsomeip::major_version_t major, vsomeip::minor_version_t minor) {
    _application->request_service(service, instance, major, minor);
}

void application::release_service(vsomeip::service_t service, vsomeip::instance_t instance) {
    _application->release_service(service, instance);
}
void application::request_event(
        vsomeip::service_t service,
        vsomeip::instance_t instance,
        vsomeip::event_t event,
        std::set<vsomeip::eventgroup_t> const& event_groups,
        vsomeip::event_type_e type,
        vsomeip::reliability_type_e reliability)
{
    _application->request_event(service, instance, event, event_groups, type, reliability);
}

void application::release_event(
        vsomeip::service_t service,
        vsomeip::instance_t instance,
        vsomeip::event_t event)
{
    _application->release_event(service, instance, event);
}

void application::subscribe(
        vsomeip::service_t service,
        vsomeip::instance_t instance,
        vsomeip::eventgroup_t event_group,
        vsomeip::major_version_t major,
        vsomeip::event_t event)
{
    _application->subscribe(service, instance, event_group, major, event);
}

void application::subscribe_with_debounce(
        vsomeip::service_t service,
        vsomeip::instance_t instance,
        vsomeip::eventgroup_t event_group,
        vsomeip::major_version_t major,
        vsomeip::event_t event,
        vsomeip::debounce_filter_t const& filter)
{
    _application->subscribe_with_debounce(service, instance, event_group, major, event, filter);
}

void application::unsubscribe(
        vsomeip::service_t service,
        vsomeip::instance_t instance,
        vsomeip::eventgroup_t event_group)
{
    _application->unsubscribe(service, instance, event_group);
}

void application::offer_service(
        vsomeip::service_t service,
        vsomeip::instance_t instance,
        vsomeip::major_version_t major,
        vsomeip::minor_version_t minor)
{
    _application->offer_service(service, instance, major, minor);
}

void application::stop_offer_service(
        vsomeip::service_t service,
        vsomeip::instance_t instance,
        vsomeip::major_version_t major,
        vsomeip::minor_version_t minor)
{
    _application->stop_offer_service(service, instance, major, minor);
}

void application::offer_event(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::event_t notifier,
                              std::set<vsomeip::eventgroup_t> const& event_groups,
                              vsomeip::event_type_e type,
                              std::chrono::milliseconds cycle,
                              bool change_resets_cycle,
                              bool update_on_change,
                              vsomeip::epsilon_change_func_t const& epsilon_change_func,
                              vsomeip::reliability_type_e reliability)
{
    _application->offer_event(service, instance, notifier, event_groups, type, cycle, change_resets_cycle,
                              update_on_change, epsilon_change_func, reliability);
}

void application::stop_offer_event(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::event_t event)
{
    _application->stop_offer_event(service, instance, event);
}

void application::notify(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::event_t event,
                         bool force, uint8_t const* data, uint32_t data_len)
{
    auto payload = _runtime->create_payload(data, data_len);
    _application->notify(service, instance, event, payload, force);
}

void application::setup_state_handler(on_state_callback_t callback) {
    _application->register_state_handler(
    [c = std::move(callback)](vsomeip::state_type_e state) {
                c(state == vsomeip::state_type_e::ST_REGISTERED ? REGISTERED : DEREGISTERED); }
    );
}

void application::setup_avail_handler(on_avail_callback_t callback) {
    _application->register_availability_handler(
    vsomeip::ANY_SERVICE, vsomeip::ANY_INSTANCE,
    [c = std::move(callback)](vsomeip::service_t svc, vsomeip::instance_t inst, bool avail) {
                c(svc, inst, avail);}
    );
}

void application::setup_avail_handler(vsomeip::service_t service, vsomeip::instance_t instance,
                                      vsomeip::major_version_t  major, on_avail_callback_t callback)
{
    _application->register_availability_handler(service, instance,
            [c = std::move(callback)](vsomeip::service_t svc, vsomeip::instance_t inst, bool avail) {
                c(svc, inst, avail);},
                major, vsomeip::ANY_MINOR
    );
}

void application::clear_avail_handler(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::major_version_t  major)
{
    _application->unregister_availability_handler(service, instance, major);
}

void application::setup_msg_handler(on_msg_callback_t callback) {
    _application->register_message_handler(
    vsomeip::ANY_SERVICE, vsomeip::ANY_INSTANCE, vsomeip::ANY_METHOD,
    [c = std::move(callback)](std::shared_ptr<vsomeip::message> const& msg) {
                c(msg);
        });
}

vsomeip::session_t
application::send_request(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::method_t method,
                          major_version major, uint8_t const* data, uint32_t data_len, bool reliable)
{
    auto payload = _runtime->create_payload(data, data_len);
    auto msg = _runtime->create_request(reliable);
    msg->set_service(service);
    msg->set_instance(instance);
    msg->set_method(method);
    msg->set_payload(payload);
    msg->set_interface_version(major);
    _application->send(msg);
    return msg->get_session();
}

void application::send_response(service_id service, instance_id instance, method_id method,
                   client_id client, session_id session, major_version major, bool reliable,
                    vsomeip::return_code_e rc, uint8_t const* data, uint32_t data_len)
{
    auto payload = _runtime->create_payload(data, data_len);
    auto msg = _runtime->create_message(reliable);
    msg->set_service(service);
    msg->set_instance(instance);
    msg->set_method(method);
    msg->set_client(client);
    msg->set_session(session);
    msg->set_interface_version(major);
    msg->set_message_type(vsomeip::message_type_e::MT_RESPONSE);
    msg->set_return_code(rc);
    msg->set_payload(payload);
    _application->send(msg);
}

void application::send_error(service_id service, instance_id instance, method_id method, client_id client,
                             session_id session, major_version major, bool reliable, vsomeip::return_code_e rc)
{
    auto msg = _runtime->create_message(reliable);
    msg->set_service(service);
    msg->set_instance(instance);
    msg->set_method(method);
    msg->set_client(client);
    msg->set_session(session);
    msg->set_interface_version(major);
    msg->set_message_type(vsomeip::message_type_e::MT_RESPONSE);
    msg->set_return_code(rc);
    _application->send(msg);
}

std::shared_ptr<vsomeip::message> application::create_message() {
    return _runtime->create_message();
}
