// (C) 2024 BMW AG. All rights reserved.

use std::fmt;
use super::VSomeipPayload;

macro_rules! base_type {
    ($name:ident, $base_type:ty) => {
        #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
        pub struct $name (pub $base_type);

        impl $name {
            pub fn id(&self) -> $base_type { self.0 }
        }

        impl From<$base_type> for $name {
            fn from(id: $base_type) -> Self { Self(id) }
        }
    };

    ($name:ident, $base_type:ty, $display_fmt:expr) => {
        base_type!($name, $base_type);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, $display_fmt, self.0)
            }
        }
    };
}

base_type!(ServiceID, u16, "{:04x}");
pub const ANY_SERVICE: ServiceID = ServiceID(0xffff);

base_type!(InstanceID, u16, "{:04x}");
pub const ANY_INSTANCE: InstanceID = InstanceID(0xffff);

base_type!(MethodID, u16, "{:04x}");
pub const ANY_METHOD: MethodID = MethodID(0xffff);

base_type!(EventGroupID, u16, "{:04x}");

base_type!(SessionID, u16, "{:04x}");
pub const NO_SESSION: SessionID = SessionID(0x0000);

base_type!(ClientID, u16, "{:04x}");
pub const UNKNOWN_CLIENT: ClientID = ClientID(0x0000);

base_type!(MajorVersion, u8);
pub const ANY_MAJOR_VERSION: MajorVersion = MajorVersion(0xff);

base_type!(MinorVersion, u32);
pub const ANY_MINOR_VERSION: MinorVersion = MinorVersion(0xffff_ffff);

base_type!(ProtocolVersion, u8);

/// Version (major, minor) for service interfaces
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy)]
pub struct InterfaceVersion {
    pub major: MajorVersion,
    pub minor: MinorVersion,
}

impl InterfaceVersion {
    /// Returns ANY_MAJOR_VERSION.ANY_MINOR_VERSION.
    pub fn make_any() -> Self {
        InterfaceVersion{ major: ANY_MAJOR_VERSION, minor: ANY_MINOR_VERSION }
    }

    /// Returns the major.minor version.
    pub fn make_version(major: u8, minor: u32) -> Self {
        InterfaceVersion{ major: MajorVersion(major), minor: MinorVersion(minor) }
    }

    /// Returns the major.ANY_MINOR_VERSION.
    pub fn make_major(major: u8) -> Self {
        InterfaceVersion{ major: MajorVersion(major), minor: ANY_MINOR_VERSION }
    }
}

impl fmt::Display for InterfaceVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.major == ANY_MAJOR_VERSION {
            write!(f, "-.-")
        } else {
            if self.minor == ANY_MINOR_VERSION {
                write!(f, "{}.-", self.major.id())
            } else {
                write!(f, "{}.{}", self.major.id(), self.minor.id())
            }
        }
    }
}

/// Common elements of every SOME/IP message received or sent by vsomeip.
/// Not all elements are always meaningful or required.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct MessageHeader {
    /// ID of the service interface (mandatory)
    pub service_id: ServiceID,
    /// ID of the service instance (mandatory)
    pub instance_id: InstanceID,
    /// ID of the method within the service interface (mandatory)
    /// In case of events this is also called "NotifierID"
    pub method_id: MethodID,
    /// SOME/IP Client ID of the application
    /// For message to be sent vsomeip will automatically insert this - applications can set it to UNKNOWN_CLIENT
    pub client_id: ClientID,
    /// SOME/IP Session ID
    /// For NOTIFICATION and REQUEST_NO_RETURN this field is irrelevant.
    /// For REQUEST vsomeip will generate the SessionID and return it when sending the message.
    /// For RESPONSE and ERROR messages the session ID must be the same as the triggering REQUEST.
    pub session_id: SessionID,
    /// Service Interface version. Not relevant in send-direction.
    /// In receive direction only the major version is indicated, because the minor version is not
    /// contained in SOME/IP messages directly.
    pub interface_version: InterfaceVersion,
    /// Indicates whether the message was sent on reliable transport (TCP) or not (UDP).
    pub reliable: bool,
}

impl fmt::Display for MessageHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}-{} ({}:{})", self.service_id, self.instance_id, self.method_id,
            self.interface_version, self.client_id, self.session_id)
    }
}

/// message types corresponding to the SOME/IP message types on the wire
#[derive(Debug)]
pub enum MessageType {
    /// Request message requiring a response returned back
    Request{ header: MessageHeader, data: VSomeipPayload },
    /// Request message without response (fire-and-forget)
    RequestNoReturn{ header: MessageHeader, data: VSomeipPayload },
    /// Response to a Request message
    Response{ header: MessageHeader, data: VSomeipPayload },
    /// Error message for exceptional cases where no Response can be sent
    Error{ header: MessageHeader, return_code: ReturnCode, data: VSomeipPayload },
    /// Event notification (after consumer subscribed to the event)
    Notification{ header: MessageHeader, is_initial: bool, data: VSomeipPayload },
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Request{header, data} =>
                write!(f, "REQUEST {}: [{:?}]", header, data.as_bytes_ref()),
            MessageType::RequestNoReturn{ header, data} =>
                write!(f, "REQUEST(FF) {}: [{:?}]", header, data.as_bytes_ref()),
            MessageType::Response{ header, data } =>
                write!(f, "RESPONSE {}: [{:?}]", header,  data.as_bytes_ref()),
            MessageType::Error{ header, return_code, data} =>
                write!(f, "RESPONSE {} ({}): [{:?}]", header, return_code, data.as_bytes_ref()),
            MessageType::Notification{ header, is_initial: _is_initial, data} =>
                write!(f, "NOTIFICATION {}: [{:?}]", header, data.as_bytes_ref()),
        }
    }
}

/// return codes corresponding to SOME/IP return code
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum ReturnCode {
    Ok,
    NotOk,
    UnknownService,
    UnknownMethod,
    NotReady,
    NotReachable,
    Timeout,
    WrongProtocolVersion,
    WrongInterfaceVersion,
    MalformedMessage,
    WrongMessageType,
    Unknown,
}

impl ReturnCode {

    /// Returns whether an application is allowed to send the return code in a response.
    pub fn can_be_sent(&self) -> bool {
        match self {
            ReturnCode::NotReachable => false,
            ReturnCode::Timeout => false,
            ReturnCode::UnknownService => false,
            ReturnCode::WrongInterfaceVersion => false,
            ReturnCode::WrongProtocolVersion => false,
            _ => true
        }
    }
}

impl fmt::Display for ReturnCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReturnCode::Ok => write!(f, "OK"),
            ReturnCode::NotOk => write!(f, "NOT_OK"),
            ReturnCode::UnknownService => write!(f, "UNKNOWN_SERVICE"),
            ReturnCode::UnknownMethod => write!(f, "UNKNOWN_METHOD"),
            ReturnCode::NotReady => write!(f, "NOT_READY"),
            ReturnCode::NotReachable => write!(f, "NOT_REACHABLE"),
            ReturnCode::Timeout => write!(f, "TIMEOUT"),
            ReturnCode::WrongProtocolVersion => write!(f, "WRONG_PROTOCOL_VERSION"),
            ReturnCode::WrongInterfaceVersion => write!(f, "WRONG_INTERFACE_VERSION"),
            ReturnCode::MalformedMessage => write!(f, "MALFORMED_MESSAGE"),
            ReturnCode::WrongMessageType => write!(f, "WRONG_MESSAGE_TYPE"),
            ReturnCode::Unknown => write!(f, "UNKNOWN")
        }
    }
}


#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Reliability {
    Reliable,
    Unreliable,
    Both,
    Unknown,
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn service_id_fmt_test() {
        assert_eq!("44a2", format!("{}", ServiceID::from(0x44a2)));
    }

    #[test]
    fn service_id_eq_test() {
        assert_eq!(ANY_SERVICE, ServiceID::from(0xffff));
        assert_eq!(ServiceID(2), ServiceID::from(2));
        assert_ne!(ServiceID(0x23), ServiceID::from(23));
    }
}
