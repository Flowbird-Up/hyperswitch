use common_enums::{CountryAlpha2, MerchantProductType};
use common_utils::{id_type, pii};
use masking::Secret;
use strum::EnumString;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum SetMetaDataRequest {
    ProductionAgreement(ProductionAgreementRequest),
    SetupProcessor(SetupProcessor),
    ConfigureEndpoint,
    SetupComplete,
    FirstProcessorConnected(ProcessorConnected),
    SecondProcessorConnected(ProcessorConnected),
    ConfiguredRouting(ConfiguredRouting),
    TestPayment(TestPayment),
    IntegrationMethod(IntegrationMethod),
    ConfigurationType(ConfigurationType),
    IntegrationCompleted,
    SPRoutingConfigured(ConfiguredRouting),
    Feedback(Feedback),
    ProdIntent(ProdIntent),
    SPTestPayment,
    DownloadWoocom,
    ConfigureWoocom,
    SetupWoocomWebhook,
    IsMultipleConfiguration,
    #[serde(skip)]
    IsChangePasswordRequired,
    OnboardingSurvey(OnboardingSurvey),
    ReconStatus(ReconStatus),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ProductionAgreementRequest {
    pub version: String,
    #[serde(skip_deserializing)]
    pub ip_address: Option<Secret<String, pii::IpAddress>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SetupProcessor {
    pub connector_id: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ProcessorConnected {
    pub processor_id: id_type::MerchantConnectorAccountId,
    pub processor_name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct OnboardingSurvey {
    pub designation: Option<String>,
    pub about_business: Option<String>,
    pub business_website: Option<String>,
    pub hyperswitch_req: Option<String>,
    pub major_markets: Option<Vec<String>>,
    pub business_size: Option<String>,
    pub required_features: Option<Vec<String>>,
    pub required_processors: Option<Vec<String>>,
    pub planned_live_date: Option<String>,
    pub miscellaneous: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ConfiguredRouting {
    pub routing_id: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TestPayment {
    pub payment_id: id_type::PaymentId,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct IntegrationMethod {
    pub integration_type: String,
}
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub enum ConfigurationType {
    Single,
    Multiple,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Feedback {
    pub email: pii::Email,
    pub description: Option<String>,
    pub rating: Option<i32>,
    pub category: Option<String>,
}
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct ProdIntent {
    pub legal_business_name: Option<String>,
    pub business_label: Option<String>,
    pub business_location: Option<CountryAlpha2>,
    pub display_name: Option<String>,
    pub poc_email: Option<Secret<String>>,
    pub business_type: Option<String>,
    pub business_identifier: Option<String>,
    pub business_website: Option<String>,
    pub poc_name: Option<Secret<String>>,
    pub poc_contact: Option<Secret<String>>,
    pub comments: Option<String>,
    pub is_completed: bool,
    #[serde(default)]
    pub product_type: MerchantProductType,
    pub business_country_name: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct ReconStatus {
    pub is_order_data_set: bool,
    pub is_processor_data_set: bool,
}

#[derive(Debug, serde::Deserialize, EnumString, serde::Serialize)]
pub enum GetMetaDataRequest {
    ProductionAgreement,
    SetupProcessor,
    ConfigureEndpoint,
    SetupComplete,
    FirstProcessorConnected,
    SecondProcessorConnected,
    ConfiguredRouting,
    TestPayment,
    IntegrationMethod,
    ConfigurationType,
    IntegrationCompleted,
    StripeConnected,
    PaypalConnected,
    SPRoutingConfigured,
    Feedback,
    ProdIntent,
    SPTestPayment,
    DownloadWoocom,
    ConfigureWoocom,
    SetupWoocomWebhook,
    IsMultipleConfiguration,
    IsChangePasswordRequired,
    OnboardingSurvey,
    ReconStatus,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct GetMultipleMetaDataPayload {
    pub results: Vec<GetMetaDataRequest>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GetMultipleMetaDataRequest {
    pub keys: String,
}

#[derive(Debug, serde::Serialize)]
pub enum GetMetaDataResponse {
    ProductionAgreement(bool),
    SetupProcessor(Option<SetupProcessor>),
    ConfigureEndpoint(bool),
    SetupComplete(bool),
    FirstProcessorConnected(Option<ProcessorConnected>),
    SecondProcessorConnected(Option<ProcessorConnected>),
    ConfiguredRouting(Option<ConfiguredRouting>),
    TestPayment(Option<TestPayment>),
    IntegrationMethod(Option<IntegrationMethod>),
    ConfigurationType(Option<ConfigurationType>),
    IntegrationCompleted(bool),
    StripeConnected(Option<ProcessorConnected>),
    PaypalConnected(Option<ProcessorConnected>),
    SPRoutingConfigured(Option<ConfiguredRouting>),
    Feedback(Option<Feedback>),
    ProdIntent(Option<ProdIntent>),
    SPTestPayment(bool),
    DownloadWoocom(bool),
    ConfigureWoocom(bool),
    SetupWoocomWebhook(bool),
    IsMultipleConfiguration(bool),
    IsChangePasswordRequired(bool),
    OnboardingSurvey(Option<OnboardingSurvey>),
    ReconStatus(Option<ReconStatus>),
}
