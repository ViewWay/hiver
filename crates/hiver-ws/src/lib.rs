//! hiver-ws — Spring Web Services — SOAP/WSDL contract-first web services
//! hiver-ws — SOAP/WSDL 契约优先 Web 服务
//!
//! # Overview / 概述
//!
//! `hiver-ws` provides SOAP web services with contract-first development,
//! message dispatching, XML marshalling, WSDL generation, and WS-Security.
//! Equivalent to: Spring Web Services
//!
//! `hiver-ws` 提供SOAP Web服务，支持契约优先开发、消息调度、XML编组、WSDL生成和WS-Security。
//! 等价于: Spring Web Services
//!
//! # Features / 功能
//!
//! - SoapMessage/SoapEnvelope/SoapFault
//! - `MessageDispatcher` with endpoint registration
//! - Endpoint trait with PayloadRoot/SoapAction mapping
//! - XML marshalling/unmarshalling
//! - WSDL generation from XSD
//! - WS-Security (message signing, authentication)

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;

pub mod dispatcher;
pub mod endpoint;
pub mod marshalling;
pub mod security;
pub mod soap;
pub mod transport;
pub mod wsdl;

// Re-exports
pub use dispatcher::MessageDispatcher;
pub use endpoint::{Endpoint, PayloadRoot, SoapAction};
pub use marshalling::{DefaultMarshaller, MarshalError, XmlMarshal, XmlUnmarshal};
pub use security::{SecurityConfig, WsSecurityHeader};
pub use soap::{SoapBody, SoapEnvelope, SoapFault, SoapHeader, SoapMessage};
pub use transport::{HttpTransport, SoapRequest, SoapResponse, Transport};
pub use wsdl::{WsdlDefinition, WsdlGenerator, WsdlOperation};
