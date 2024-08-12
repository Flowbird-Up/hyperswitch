use std::fmt::Debug;
use error_stack::{report, ResultExt};
use http::StatusCode;
use serde::Deserialize;
use common_utils::ext_traits::ValueExt;
use common_utils::pii::SecretSerdeValue;
use diesel_models::enums;
use masking::ExposeInterface;
use transformers as archipel;
use crate::{
    configs::settings,
    core::errors::{self, CustomResult},
    events::connector_api_logs::ConnectorEvent,
    headers,
    services::{self, ConnectorIntegration, ConnectorValidation, request::{self}},
    types::{
        self,
        api::{self, ConnectorCommon, ConnectorCommonExt},
        ErrorResponse, RequestContent,
        Response
    },
    utils::BytesExt,
    connector::utils::RouterData
};

pub mod transformers;

#[derive(Debug, Clone)]
pub struct Archipel;

impl api::Payment for Archipel {}
impl api::PaymentSession for Archipel {}
impl api::ConnectorAccessToken for Archipel {}
impl api::MandateSetup for Archipel {}
impl api::PaymentAuthorize for Archipel {}
impl api::PaymentSync for Archipel {}
impl api::PaymentCapture for Archipel {}
impl api::PaymentVoid for Archipel {}
impl api::Refund for Archipel {}
impl api::RefundExecute for Archipel {}
impl api::RefundSync for Archipel {}
impl api::PaymentToken for Archipel {}
impl api::PaymentIncrementalAuthorization for Archipel {}


impl<Flow, Request, Response> ConnectorCommonExt<Flow, Request, Response> for Archipel
    where
        Self: ConnectorIntegration<Flow, Request, Response>,{
    fn build_headers(
        &self,
        req: &types::RouterData<Flow, Request, Response>,
        _connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, request::Maskable<String>)>, errors::ConnectorError> {
        let mut header = vec![(
            headers::CONTENT_TYPE.to_string(),
            self.get_content_type().to_string().into(),
        )];
        let mut api_key = self.get_auth_header(&req.connector_auth_type)?;
        header.append(&mut api_key);
        Ok(header)
    }
}

impl ConnectorCommon for Archipel {
    fn id(&self) -> &'static str {
        "archipel"
    }

    fn get_currency_unit(&self) -> api::CurrencyUnit {
        //    TODO! Check connector documentation, on which unit they are processing the currency.
        //    If the connector accepts amount in lower unit ( i.e cents for USD) then return api::CurrencyUnit::Minor,
        //    if connector accepts amount in base unit (i.e dollars for USD) then return api::CurrencyUnit::Base
        api::CurrencyUnit::Base
    }

    fn get_auth_header(&self, _auth_type:&types::ConnectorAuthType)-> CustomResult<Vec<(String,request::Maskable<String>)>,errors::ConnectorError> {
        Ok(vec![])
    }

    fn common_get_content_type(&self) -> &'static str {
        "application/json"
    }

    fn base_url<'a>(&self, connectors: &'a settings::Connectors) -> &'a str {
        connectors.archipel.base_url.as_ref()
    }

    fn build_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        let response: archipel::ArchipelErrorResponse =
            archipel::ArchipelErrorResponse {
                status_code: res.status_code,
                code: String::new(),
                message: serde_json::from_slice(&res.response).unwrap_or("").to_string(),
                reason: Some(StatusCode::from_u16(res.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR).to_string())
            };

        event_builder.map(|i| i.set_response_body(&response.message));
        router_env::logger::info!(connector_response=?response);

        Ok(ErrorResponse {
            status_code: response.status_code,
            code: response.code,
            message: response.message,
            reason: response.reason,
            attempt_status: Some(enums::AttemptStatus::Failure),
            connector_transaction_id: None,
        })
    }
}

impl ConnectorValidation for Archipel {
    // Allowed Capture methods for archipel connector
    fn validate_capture_method(&self,
                               capture_method: Option<enums::CaptureMethod>,
                               _pmt: Option<enums::PaymentMethodType>) -> CustomResult<(), errors::ConnectorError> {
        let capture_method = capture_method.unwrap_or_default();
        match capture_method {
            enums::CaptureMethod::Automatic | enums::CaptureMethod::Manual => Ok(()),
            | enums::CaptureMethod::ManualMultiple
            | enums::CaptureMethod::Scheduled => {
                Err(errors::ConnectorError::NotSupported {
                    message: capture_method.to_string(),
                    connector: self.id(),
                }
                    .into())
            }
        }
    }

    fn validate_mandate_payment(
        &self,
        pm_type: Option<enums::PaymentMethodType>,
        pm_data: types::domain::payments::PaymentMethodData,
    ) -> CustomResult<(), errors::ConnectorError> {
        let mandate_supported_pmd = std::collections::HashSet::from([
            crate::connector::utils::PaymentMethodDataType::Card,
            // add other PaymentMethodDataType for mandate
        ]);
        crate::connector::utils::is_mandate_supported(pm_data, pm_type, mandate_supported_pmd, self.id())
    }
}

#[derive(Debug, Deserialize)]
struct ConnectorMetadata {
    tenant_id: Option<String>
}

impl Default for ConnectorMetadata {
    fn default() -> Self { 
        ConnectorMetadata { 
            tenant_id: None
        } 
    }
}

fn get_tenant_id(connector_metadata: SecretSerdeValue) -> Result<String, errors::ConnectorError> {
    let connector_meta: ConnectorMetadata = serde_json::from_value(connector_metadata.expose())
        .unwrap_or(ConnectorMetadata::default());
    // TODO: remove debug log
    router_env::debug!(archipel_tenant_id=format!("{:?}", connector_meta));
    if !connector_meta.tenant_id.is_none() {
        Ok(connector_meta.tenant_id.unwrap())
    }
    else {
        Err(errors::ConnectorError::InvalidConnectorConfig {
            config: "Missing tenant_id. Please check your merchant connector account metadata."
        })
    }
}

impl ConnectorIntegration<api::Authorize,
    types::PaymentsAuthorizeData,
    types::PaymentsResponseData, > for Archipel {
    fn get_headers(&self,
                   req: &types::PaymentsAuthorizeRouterData,
                   connectors: &settings::Connectors,) -> CustomResult<Vec<(String, request::Maskable<String>)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(&self,
               req: &types::PaymentsAuthorizeRouterData,
               connectors: &settings::Connectors, ) -> CustomResult<String, errors::ConnectorError> {
        let capture_method = req.request.capture_method.ok_or(errors::ConnectorError::CaptureMethodNotSupported)?;
        match capture_method {
            enums::CaptureMethod::Automatic => {
                Ok(format!("{}{}", self.base_url(connectors), "Transaction/v1/pay"))
            },
            enums::CaptureMethod::Manual => {
                Ok(format!("{}{}", self.base_url(connectors), "Transaction/v1/authorize"))
            }
            enums::CaptureMethod::ManualMultiple
            | enums::CaptureMethod::Scheduled => {
                Err(report!(errors::ConnectorError::CaptureMethodNotSupported))
            }
        }
    }

    fn get_request_body(&self,
                        req: &types::PaymentsAuthorizeRouterData,
                        _connectors: &settings::Connectors,) -> CustomResult<RequestContent, errors::ConnectorError> {
        let tenant = get_tenant_id(req.get_connector_meta()?)?;
        let connector_router_data =
            archipel::ArchipelRouterData::try_from((
                &self.get_currency_unit(),
                req.request.currency,
                req.request.amount,
                req,
                tenant
            ))?;
        let connector_req = archipel::ArchipelAuthorizationRequest::try_from(&connector_router_data)?;
        Ok(RequestContent::Json(Box::new(connector_req)))
    }

    fn build_request(&self,
                     req: &types::PaymentsAuthorizeRouterData,
                     connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Post)
                .url(&types::PaymentsAuthorizeType::get_url(
                    self, req, connectors,
                )?)
                .attach_default_headers()
                .headers(types::PaymentsAuthorizeType::get_headers(
                    self, req, connectors,
                )?)
                .set_body(types::PaymentsAuthorizeType::get_request_body(self, req, connectors)?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &types::PaymentsAuthorizeRouterData,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<types::PaymentsAuthorizeRouterData,errors::ConnectorError> {
        let response: archipel::ArchipelPaymentsResponse = res
            .response
            .parse_struct("PaymentsAuthorizeResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        event_builder.map(|i| i.set_response_body(&response));
        router_env::logger::info!(connector_response=?response);
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(&self,
                          res: Response,
                          event_builder: Option<&mut ConnectorEvent>) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }

    fn get_5xx_error_response(&self, res: Response, event_builder: Option<&mut ConnectorEvent>) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        let response: archipel::ArchipelErrorMessage = res
            .response
            .parse_struct("ArchipelErrorMessage")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;

        event_builder.map(|i| i.set_response_body(&serde_json::from_slice(&res.response).unwrap_or("").to_string()));
        router_env::logger::info!(connector_response=?response);

        Ok(ErrorResponse {
            status_code: res.status_code,
            code: response.code,
            message: String::new(),
            reason: response.description,
            attempt_status: Some(enums::AttemptStatus::Failure),
            connector_transaction_id: None,
        })
    }
}

impl ConnectorIntegration<api::IncrementalAuthorization,
    types::PaymentsIncrementalAuthorizationData,
    types::PaymentsResponseData, > for Archipel
{
    fn get_headers(
        &self,
        req: &types::PaymentsIncrementalAuthorizationRouterData,
        connectors: &settings::Connectors
    ) -> CustomResult<Vec<(String, request::Maskable<String>)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        req: &types::PaymentsIncrementalAuthorizationRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        let connector_payment_id = req.request.connector_transaction_id.clone();
        Ok(format!("{}{}{}",
                   self.base_url(connectors),
                   "Transaction/v1/incrementAuthorization/",
                   connector_payment_id
        ))
    }

    fn get_request_body(
        &self,
        req: &types::PaymentsIncrementalAuthorizationRouterData,
        _connectors: &settings::Connectors,
    ) -> CustomResult<RequestContent, errors::ConnectorError> {
        let connector_router_data = archipel::ArchipelRouterData::try_from((
            &self.get_currency_unit(),
            req.request.currency,
            req.request.additional_amount,
            req,
            get_tenant_id(req.get_connector_meta()?)?
        ))?;
        let connector_request =
            archipel::ArchipelIncrementalAuthorizationRequest::try_from(
                &connector_router_data,
            )?;
        Ok(RequestContent::Json(Box::new(connector_request)))
    }
    fn build_request(
        &self,
        req: &types::PaymentsIncrementalAuthorizationRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Post)
                .url(&types::IncrementalAuthorizationType::get_url(
                    self, req, connectors,
                )?)
                .attach_default_headers()
                .headers(types::IncrementalAuthorizationType::get_headers(
                    self, req, connectors,
                )?)
                .set_body(types::IncrementalAuthorizationType::get_request_body(
                    self, req, connectors,
                )?)
                .build(),
        ))
    }
    fn handle_response(
        &self,
        data: &types::PaymentsIncrementalAuthorizationRouterData,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<types::PaymentsIncrementalAuthorizationRouterData, errors::ConnectorError> {
        let response: archipel::ArchipelPaymentsResponse = res
            .response
            .parse_struct("ArchipelPaymentsResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        event_builder.map(|i| i.set_response_body(&response));
        router_env::logger::info!(connector_response=?response);
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }
    fn get_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl ConnectorIntegration<api::PSync,
    types::PaymentsSyncData,
    types::PaymentsResponseData> for Archipel {
    fn get_headers(
        &self,
        req: &types::PaymentsSyncRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, request::Maskable<String>)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        req: &types::PaymentsSyncRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        let metadata: archipel::ArchipelTransactionMetadata = req.request.connector_meta.clone()
            .unwrap()
            .parse_value("ArchipelTransactionMetadata")
            .change_context(errors::ConnectorError::MissingConnectorTransactionID)?;
        Ok(format!("{}{}{}", self.base_url(connectors),
                   "Transaction/v1/transactions/",
                   metadata.transaction_id)
        )
    }

    fn build_request(
        &self,
        req: &types::PaymentsSyncRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Get)
                .url(&types::PaymentsSyncType::get_url(self, req, connectors)?)
                .attach_default_headers()
                .headers(types::PaymentsSyncType::get_headers(self, req, connectors)?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &types::PaymentsSyncRouterData,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<types::PaymentsSyncRouterData, errors::ConnectorError> {
        let response: archipel::ArchipelPaymentsResponse = res
            .response
            .parse_struct("ArchipelPaymentsResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        event_builder.map(|i| i.set_response_body(&response));
        router_env::logger::info!(connector_response=?response);
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl ConnectorIntegration<api::Capture,
    types::PaymentsCaptureData,
    types::PaymentsResponseData, > for Archipel {
    fn get_headers(
        &self,
        req: &types::PaymentsCaptureRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, request::Maskable<String>)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        req: &types::PaymentsCaptureRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        Ok(format!("{}{}{}", self.base_url(connectors),
                "Transaction/v1/capture/",
                req.request.connector_transaction_id)
        )
    }

    fn get_request_body(
        &self,
        req: &types::PaymentsCaptureRouterData,
        _connectors: &settings::Connectors,
    ) -> CustomResult<RequestContent, errors::ConnectorError> {
        let connector_router_data =
            archipel::ArchipelRouterData::try_from((
                &self.get_currency_unit(),
                req.request.currency,
                req.request.amount_to_capture,
                req,
                get_tenant_id(req.get_connector_meta()?)?
            ))?;
        let connector_req = archipel::ArchipelCaptureRequest::try_from(&connector_router_data)?;
        Ok(RequestContent::Json(Box::new(connector_req)))
    }

    fn build_request(
        &self,
        req: &types::PaymentsCaptureRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Post)
                .url(&types::PaymentsCaptureType::get_url(self, req, connectors)?)
                .attach_default_headers()
                .headers(types::PaymentsCaptureType::get_headers(
                    self, req, connectors,
                )?)
                .set_body(types::PaymentsCaptureType::get_request_body(self, req, connectors)?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &types::PaymentsCaptureRouterData,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<types::PaymentsCaptureRouterData, errors::ConnectorError> {
        let response: archipel::ArchipelPaymentsResponse = res
            .response
            .parse_struct("ArchipelPaymentsResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        event_builder.map(|i| i.set_response_body(&response));
        router_env::logger::info!(connector_response=?response);
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl ConnectorIntegration<api::SetupMandate,
    types::SetupMandateRequestData,
    types::PaymentsResponseData, > for Archipel {
    fn get_headers(&self,
                   req: &types::SetupMandateRouterData,
                   connectors: &settings::Connectors,) -> CustomResult<Vec<(String, request::Maskable<String>)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(&self,
               _req: &types::SetupMandateRouterData,
               connectors: &settings::Connectors, ) -> CustomResult<String, errors::ConnectorError> {
        Ok(format!("{}{}", self.base_url(connectors), "Transaction/v1/verify"))
    }

    fn get_request_body(&self,
                        req: &types::SetupMandateRouterData,
                        _connectors: &settings::Connectors,) -> CustomResult<RequestContent, errors::ConnectorError> {
        let connector_router_data =
            archipel::ArchipelRouterData::try_from((
                &self.get_currency_unit(),
                req.request.currency,
                0,
                req,
                get_tenant_id(req.get_connector_meta()?)?
            ))?;
        let connector_req = archipel::ArchipelAuthorizationRequest::try_from(&connector_router_data)?;
        Ok(RequestContent::Json(Box::new(connector_req)))
    }

    fn build_request(
        &self,
        req: &types::SetupMandateRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Post)
                .url(&types::SetupMandateType::get_url(self, req, connectors)?)
                .attach_default_headers()
                .headers(types::SetupMandateType::get_headers(self, req, connectors)?)
                .set_body(types::SetupMandateType::get_request_body(self, req, connectors)?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &types::SetupMandateRouterData,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<types::SetupMandateRouterData, errors::ConnectorError> {
        let response: archipel::ArchipelPaymentsResponse = res
            .response
            .parse_struct("ArchipelPaymentsResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        event_builder.map(|i| i.set_response_body(&response));
        router_env::logger::info!(connector_response=?response);
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl ConnectorIntegration<api::Execute,
    types::RefundsData,
    types::RefundsResponseData, > for Archipel {
    fn get_headers(&self, req: &types::RefundsRouterData<api::Execute>, connectors: &settings::Connectors,) -> CustomResult<Vec<(String,request::Maskable<String>)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(&self, _req: &types::RefundsRouterData<api::Execute>, _connectors: &settings::Connectors,) -> CustomResult<String,errors::ConnectorError> {
        Err(errors::ConnectorError::NotImplemented("get_url method".to_string()).into())
    }

    fn get_request_body(&self, req: &types::RefundsRouterData<api::Execute>, _connectors: &settings::Connectors,) -> CustomResult<RequestContent, errors::ConnectorError> {
        let connector_router_data =
            archipel::ArchipelRouterData::try_from((
                &self.get_currency_unit(),
                req.request.currency,
                req.request.refund_amount,
                req,
                get_tenant_id(req.get_connector_meta()?)?
            ))?;
        let connector_req = archipel::ArchipelRefundRequest::try_from(&connector_router_data)?;
        Ok(RequestContent::Json(Box::new(connector_req)))
    }

    fn build_request(&self, req: &types::RefundsRouterData<api::Execute>, connectors: &settings::Connectors,) -> CustomResult<Option<services::Request>,errors::ConnectorError> {
        let request = services::RequestBuilder::new()
            .method(services::Method::Post)
            .url(&types::RefundExecuteType::get_url(self, req, connectors)?)
            .attach_default_headers()
            .headers(types::RefundExecuteType::get_headers(self, req, connectors)?)
            .set_body(types::RefundExecuteType::get_request_body(self, req, connectors)?)
            .build();
        Ok(Some(request))
    }

    fn handle_response(
        &self,
        data: &types::RefundsRouterData<api::Execute>,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<types::RefundsRouterData<api::Execute>,errors::ConnectorError> {
        let response: archipel::RefundResponse = res.response.parse_struct("archipel RefundResponse").change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        event_builder.map(|i| i.set_response_body(&response));
        router_env::logger::info!(connector_response=?response);
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(&self, res: Response, event_builder: Option<&mut ConnectorEvent>) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl ConnectorIntegration<api::RSync,
    types::RefundsData,
    types::RefundsResponseData> for Archipel {
    fn get_headers(&self, req: &types::RefundSyncRouterData,connectors: &settings::Connectors,) -> CustomResult<Vec<(String, request::Maskable<String>)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(&self, _req: &types::RefundSyncRouterData,_connectors: &settings::Connectors,) -> CustomResult<String,errors::ConnectorError> {
        Err(errors::ConnectorError::NotImplemented("get_url method".to_string()).into())
    }

    fn build_request(
        &self,
        req: &types::RefundSyncRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Get)
                .url(&types::RefundSyncType::get_url(self, req, connectors)?)
                .attach_default_headers()
                .headers(types::RefundSyncType::get_headers(self, req, connectors)?)
                .set_body(types::RefundSyncType::get_request_body(self, req, connectors)?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &types::RefundSyncRouterData,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<types::RefundSyncRouterData,errors::ConnectorError,> {
        let response: archipel::RefundResponse = res.response.parse_struct("archipel RefundSyncResponse").change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        event_builder.map(|i| i.set_response_body(&response));
        router_env::logger::info!(connector_response=?response);
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(&self, res: Response, event_builder: Option<&mut ConnectorEvent>) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl ConnectorIntegration<api::PaymentMethodToken,
    types::PaymentMethodTokenizationData,
    types::PaymentsResponseData, > for Archipel {
    // Not Implemented (R)
}

impl ConnectorIntegration<api::Session,
    types::PaymentsSessionData,
    types::PaymentsResponseData, > for Archipel {
    // Not Implemented (R)
}

impl ConnectorIntegration<api::AccessTokenAuth,
    types::AccessTokenRequestData,
    types::AccessToken> for Archipel {
    // Not Implemented (R)
}

impl ConnectorIntegration<api::Void,
    types::PaymentsCancelData,
    types::PaymentsResponseData, > for Archipel {
    fn get_headers(&self,
                   req: &types::PaymentsCancelRouterData,
                   connectors: &settings::Connectors
    ) -> CustomResult<Vec<(String, request::Maskable<String>)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
            self.common_get_content_type()
        }

        fn get_url(&self,
                   req: &types::PaymentsCancelRouterData,
                   connectors: &settings::Connectors) -> CustomResult<String, errors::ConnectorError> {
            Ok(format!(
                "{}{}{}",
                self.base_url(connectors),
                "Transaction/v1/cancel/",
                req.request.connector_transaction_id)
            )
        }

        fn get_request_body(&self,
                            req: &types::PaymentsCancelRouterData,
                            _connectors: &settings::Connectors,) -> CustomResult<RequestContent, errors::ConnectorError> {
            let tenant = get_tenant_id(req.get_connector_meta()?)?;
            let connector_router_data =
                archipel::ArchipelRouterData::try_from((
                    &self.get_currency_unit(),
                    req.request
                        .currency
                        .ok_or(errors::ConnectorError::MissingRequiredField {
                            field_name: "Currency",
                        })?,
                    req.request
                        .amount
                        .ok_or(errors::ConnectorError::MissingRequiredField {
                            field_name: "Amount",
                        })?,
                    req,
                    tenant
                ));
            let connector_req = archipel::ArchipelPaymentsCancelRequest::try_from(&connector_router_data?)?;
            Ok(RequestContent::Json(Box::new(connector_req)))
        }

        fn build_request(
            &self,
            req: &types::PaymentsCancelRouterData,
            connectors: &settings::Connectors,
        ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
            Ok(Some(
                services::RequestBuilder::new()
                    .method(services::Method::Post)
                    .url(&types::PaymentsVoidType::get_url(self, req, connectors)?)
                    .attach_default_headers()
                    .headers(types::PaymentsVoidType::get_headers(self, req, connectors)?)
                    .set_body(types::PaymentsVoidType::get_request_body(self, req, connectors)?)
                    .build(),
            ))
        }

        fn handle_response(
            &self,
            data: &types::PaymentsCancelRouterData,
            event_builder: Option<&mut ConnectorEvent>,
            res: Response,
        ) -> CustomResult<types::PaymentsCancelRouterData, errors::ConnectorError> {
            let response: archipel::ArchipelPaymentsResponse = res
                .response
                .parse_struct("ArchipelPaymentsResponse")
                .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
            event_builder.map(|i| i.set_response_body(&response));
            router_env::logger::info!(connector_response=?response);
            types::RouterData::try_from(types::ResponseRouterData {
                response,
                data: data.clone(),
                http_code: res.status_code,
            })
        }

        fn get_error_response(&self,
                              res: Response,
                              event_builder: Option<&mut ConnectorEvent>
        ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
            self.build_error_response(res, event_builder)
        }
    }

#[async_trait::async_trait]
impl api::IncomingWebhook for Archipel {
    fn get_webhook_object_reference_id(
        &self,
        _request: &api::IncomingWebhookRequestDetails<'_>,
    ) -> CustomResult<api::webhooks::ObjectReferenceId, errors::ConnectorError> {
        Err(report!(errors::ConnectorError::WebhooksNotImplemented))
    }

    fn get_webhook_event_type(
        &self,
        _request: &api::IncomingWebhookRequestDetails<'_>,
    ) -> CustomResult<api::IncomingWebhookEvent, errors::ConnectorError> {
        Err(report!(errors::ConnectorError::WebhooksNotImplemented))
    }

    fn get_webhook_resource_object(&self, _request: &api::IncomingWebhookRequestDetails<'_>,
    ) -> CustomResult<Box<dyn masking::ErasedMaskSerialize>, errors::ConnectorError> {
        Err(report!(errors::ConnectorError::WebhooksNotImplemented))
    }
}
