//! WSDL generation from contract-first XSD
//! 契约优先XSD的WSDL生成
//!
//! Equivalent to Spring WS WSDL generation
//! 等价于 Spring WS WSDL生成

use std::fmt::Write;

use serde::{Deserialize, Serialize};

/// WSDL definition / WSDL定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsdlDefinition {
    /// Target namespace / 目标命名空间
    pub target_namespace: String,
    /// Service name / 服务名称
    pub service_name: String,
    /// Port type name / 端口类型名称
    pub port_type_name: String,
    /// List of operations / 操作列表
    pub operations: Vec<WsdlOperation>,
    /// Optional embedded XSD types schema / 可选的内嵌XSD类型模式
    pub types_schema: Option<String>,
}

/// WSDL operation / WSDL操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsdlOperation {
    /// Operation name / 操作名称
    pub name: String,
    /// Input message name / 输入消息名称
    pub input_message: String,
    /// Output message name / 输出消息名称
    pub output_message: String,
    /// SOAP action URI / SOAP操作URI
    pub soap_action: String,
}

/// WSDL generator / WSDL生成器
pub struct WsdlGenerator {
    namespace: String,
    service: String,
    location: String,
}

impl WsdlGenerator {
    /// Create a new WSDL generator / 创建新的WSDL生成器
    pub fn new(namespace: &str, service: &str, location: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            service: service.to_string(),
            location: location.to_string(),
        }
    }

    /// Generate WSDL with operations / 生成包含操作的WSDL
    pub fn generate(&self, operations: Vec<WsdlOperation>) -> WsdlDefinition {
        WsdlDefinition {
            target_namespace: self.namespace.clone(),
            service_name: format!("{}Service", self.service),
            port_type_name: format!("{}Port", self.service),
            operations,
            types_schema: None,
        }
    }

    /// Generate WSDL as XML string / 生成WSDL的XML字符串
    pub fn to_xml(&self, def: &WsdlDefinition) -> String {
        let mut ops_xml = String::new();
        for op in &def.operations {
            let _ = write!(
                ops_xml,
                r#"  <wsdl:operation name="{}">
    <soap:operation soapAction="{}"/>
    <wsdl:input message="tns:{}"/>
    <wsdl:output message="tns:{}"/>
  </wsdl:operation>
"#,
                escape_xml(&op.name),
                escape_xml(&op.soap_action),
                escape_xml(&op.input_message),
                escape_xml(&op.output_message)
            );
        }

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<wsdl:definitions xmlns:wsdl="http://schemas.xmlsoap.org/wsdl/"
  xmlns:soap="http://schemas.xmlsoap.org/wsdl/soap/"
  xmlns:tns="{}"
  targetNamespace="{}">
  <wsdl:portType name="{}">
{}
  </wsdl:portType>
  <wsdl:binding name="{}Binding" type="tns:{}">
    <soap:binding transport="http://schemas.xmlsoap.org/soap/http"/>
  </wsdl:binding>
  <wsdl:service name="{}">
    <wsdl:port name="{}Port" binding="tns:{}Binding">
      <soap:address location="{}"/>
    </wsdl:port>
  </wsdl:service>
</wsdl:definitions>"#,
            escape_xml(&self.namespace),
            escape_xml(&self.namespace),
            escape_xml(&def.port_type_name),
            ops_xml,
            escape_xml(&def.port_type_name),
            escape_xml(&def.port_type_name),
            escape_xml(&def.service_name),
            escape_xml(&def.port_type_name),
            escape_xml(&def.port_type_name),
            escape_xml(&self.location)
        )
    }
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_wsdl_generation() {
        let generator =
            WsdlGenerator::new("http://example.com/ws", "User", "http://localhost:8080/ws");
        let ops = vec![WsdlOperation {
            name: "GetUser".into(),
            input_message: "GetUserRequest".into(),
            output_message: "GetUserResponse".into(),
            soap_action: "urn:GetUser".into(),
        }];
        let def = generator.generate(ops);
        let xml = generator.to_xml(&def);
        assert!(xml.contains("GetUser"));
        assert!(xml.contains("wsdl:definitions"));
    }
}
