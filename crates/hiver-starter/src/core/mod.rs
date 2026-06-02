//! 核心模块 / Core Module
//!
//! 包含自动配置、IoC 容器、组件扫描等核心功能。
//! Contains auto-configuration, IoC container, component scanning, etc.

pub mod autoconfig;
pub mod autoconfigure;
pub mod autoconfigure_processor;
pub mod bean_factory_post_processor;
pub mod bean_post_processor;
pub mod condition;
pub mod condition_evaluator;
pub mod config;
pub mod container;
pub mod loader;
pub mod logging;
pub mod registry;
pub mod scanner;

// 重新导出常用类型
// Re-export commonly used types
pub use autoconfig::{AutoConfiguration, AutoConfigurationMetadata, order};

pub use autoconfigure::{
    AutoConfigurationEntry, AutoConfigurationRegistry as ConditionalRegistry, AutoConfigureOrder,
    Condition, ConditionalOnBeanCondition, ConditionalOnClass, ConditionalOnMissingBeanCondition,
    ConditionalOnMissingClass, ConditionalOnPropertyCondition, EnableAutoConfiguration,
};

pub use autoconfigure_processor::{
    AutoConfigurationProcessor, ConditionContext as AutoConfigurationConditionContext,
    ProcessResult, SkipReason,
};

pub use bean_factory_post_processor::{
    BeanFactoryPostProcessor, ConfigurationPropertiesBinder,
    PostProcessorChain as BeanFactoryPostProcessorChain, PostProcessorContext,
    PropertyPlaceholderProcessor,
};

pub use bean_post_processor::{
    AutowiredAnnotationBeanPostProcessor, BeanContext, BeanPostProcessor, BeanPostProcessorChain,
    CommonAnnotationBeanPostProcessor,
};

pub use condition_evaluator::{ApplicableConfig, ConditionEvaluator, evaluate_conditions};

pub use container::{ApplicationContext, BeanDefinition, ComponentRegistry};
pub use registry::{
    BeanDescriptor, BeanScope, PostConstruct, PreDestroy, always_true, to_bean_name,
    topological_sort,
};

pub use condition::{Conditional, ConditionalOnMissingBean, ConditionalOnProperty};
pub use scanner::ComponentScanner;

pub use config::CoreAutoConfiguration;

pub use loader::{AutoConfigurationLoader, AutoConfigurationRegistry};
