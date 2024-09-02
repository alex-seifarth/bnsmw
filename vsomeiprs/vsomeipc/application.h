#include "vsomeipc.h"

#include <vsomeip/vsomeip.hpp>

#include <memory>
#include <thread>

class application {
    std::shared_ptr<vsomeip::runtime> _runtime;
    std::shared_ptr<vsomeip::application> _application;
    std::thread _dispatch_thread;
    bool _state_connected;

    using on_state_callback_t = std::function<void(state_type_ce)>;
    using on_avail_callback_t = std::function<void(vsomeip::service_t, vsomeip::instance_t, bool)>;
    using on_msg_callback_t = std::function<void (const std::shared_ptr< vsomeip::message > &)>;

    void start();
    void stop();

public:
    application(std::shared_ptr<vsomeip::runtime> runtime, std::shared_ptr<vsomeip::application> application);
    application(application const&) = delete;
    ~application();

    [[nodiscard]]
    static std::shared_ptr<application> create(std::string const& name);

    void setup_state_handler(on_state_callback_t callback);
    void setup_avail_handler(on_avail_callback_t callback);
    void setup_msg_handler(on_msg_callback_t callback);

    void setup_avail_handler(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::major_version_t  major,
                             on_avail_callback_t callback);
    void clear_avail_handler(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::major_version_t  major);

    [[nodiscard]]
    std::shared_ptr<vsomeip::runtime>& runtime();

    [[nodiscard]]
    std::string const& name() const;

    [[nodiscard]]
    std::shared_ptr<vsomeip::payload> create_payload_empty() const;

    [[nodiscard]]
    std::shared_ptr<vsomeip::payload> create_payload(uint8_t const* data, uint32_t size);

    [[nodiscard]]
    std::shared_ptr<vsomeip::message> create_message();

    void request_service(vsomeip::service_t service, vsomeip::instance_t instance,
                         vsomeip::major_version_t major = vsomeip::ANY_MAJOR,
                         vsomeip::minor_version_t minor = vsomeip::ANY_MINOR);

    void release_service(vsomeip::service_t service, vsomeip::instance_t instance);

    void request_event(vsomeip::service_t service, vsomeip::instance_t instance,
                       vsomeip::event_t event, std::set<vsomeip::eventgroup_t> const& event_groups,
                       vsomeip::event_type_e type = vsomeip::event_type_e::ET_EVENT,
                       vsomeip::reliability_type_e reliability = vsomeip::reliability_type_e::RT_UNKNOWN);

    void release_event(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::event_t event);

    void subscribe(vsomeip::service_t service, vsomeip::instance_t instance,
                   vsomeip::eventgroup_t event_group, vsomeip::major_version_t major = vsomeip::DEFAULT_MAJOR,
                   vsomeip::event_t event = vsomeip::ANY_EVENT);

    void subscribe_with_debounce(vsomeip::service_t service, vsomeip::instance_t instance,
                                 vsomeip::eventgroup_t event_group, vsomeip::major_version_t major,
                                 vsomeip::event_t event, vsomeip::debounce_filter_t const& filter);

    void unsubscribe(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::eventgroup_t event_group);

    void offer_service(vsomeip::service_t service, vsomeip::instance_t instance,
                       vsomeip::major_version_t major = vsomeip::DEFAULT_MAJOR,
                       vsomeip::minor_version_t minor = vsomeip::DEFAULT_MINOR);

    void stop_offer_service(vsomeip::service_t service, vsomeip::instance_t instance,
                            vsomeip::major_version_t major = vsomeip::DEFAULT_MAJOR,
                            vsomeip::minor_version_t minor =vsomeip::DEFAULT_MINOR);

    void offer_event(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::event_t notifier,
                     std::set<vsomeip::eventgroup_t> const& event_groups,
                     vsomeip::event_type_e type = vsomeip::event_type_e::ET_EVENT,
                     std::chrono::milliseconds cycle = std::chrono::milliseconds::zero(),
                     bool change_resets_cycle = false,
                     bool update_on_change = true,
                     vsomeip::epsilon_change_func_t const& epsilon_change_func = nullptr,
                     vsomeip::reliability_type_e reliability = vsomeip::reliability_type_e::RT_UNKNOWN);

    void stop_offer_event(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::event_t event);

    void notify(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::event_t event,
                bool force, uint8_t const* data, uint32_t data_len);

    vsomeip::session_t send_request(vsomeip::service_t service, vsomeip::instance_t instance, vsomeip::method_t method,
                      major_version major, uint8_t const* data, uint32_t data_len, bool reliable);

    void send_response(service_id service, instance_id instance, method_id method,
            client_id client, session_id session, major_version major, bool reliable,
            vsomeip::return_code_e rc, uint8_t const* data, uint32_t data_len);

    void send_error(service_id service, instance_id instance, method_id method, client_id client, session_id session,
                    major_version major, bool reliable, vsomeip::return_code_e rc);
};