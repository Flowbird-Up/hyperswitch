use bytes::Bytes;
use rdkafka::message::ToBytes;
use serde::{Deserialize, Serialize};
use common_utils::ext_traits::Encode;
use masking::Secret;
use crate::{core::errors, types::{self, api, storage::enums, transformers::ForeignFrom, transformers::ForeignTryFrom}};
use crate::connector::utils;
use crate::connector::utils::{AddressData, AddressDetailsData, CardData, CardIssuer, RouterData};
use crate::types::domain;

//TODO: Fill the struct with respective fields
pub struct ArchipelRouterData<T> {
    pub amount: i64, // The type of amount that a connector accepts, for example, String, i64, f64, etc.
    pub router_data: T,
    pub tenant_id: String,
}

impl<T> TryFrom<(&api::CurrencyUnit, enums::Currency, i64, T, String)> for ArchipelRouterData<T> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from((_currency_unit, _currency, amount, item, tenant):
                (&api::CurrencyUnit, enums::Currency, i64, T, String), ) -> Result<Self, Self::Error> {
        //Todo :  use utils to convert the amount to the type of amount that a connector accepts
        Ok(Self {
            amount,
            router_data: item,
            tenant_id: tenant
        })
    }
}

//TODO: Fill the struct with respective fields
// Auth Struct
pub struct ArchipelAuthType {}

impl TryFrom<&types::ConnectorAuthType> for ArchipelAuthType  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(auth_type: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
        match auth_type {
            types::ConnectorAuthType::NoKey => Ok(Self {}),
            _ => Err(errors::ConnectorError::FailedToObtainAuthType.into()),
        }
    }
}


#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ArchipelPaymentInformation {
    CardPayment (CardPaymentInformation),
    WalletPayment (WalletPaymentInformation),
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct CardPaymentInformation {
    card: ArchipelCard,
    wallet: Option<ArchipelWallet>,
    three_ds: Option<Archipel3DS>
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct WalletPaymentInformation {
    card: Option<ArchipelCard>,
    wallet: ArchipelWallet,
    three_ds: Archipel3DS
}

#[derive(Debug, Default, Serialize, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ArchipelPaymentInitiator {
    #[default]
    Customer,
    Merchant,
}

#[derive(Debug, Default, Serialize, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ArchipelPaymentCertainty {
    #[default]
    Final,
    Estimated,
}
#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelOrderRequest {
    amount: i64,
    currency: String,
    certainty: ArchipelPaymentCertainty,
    initiator: ArchipelPaymentInitiator,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct CardExpiryDate {
    month: Secret<String>,
    year: Secret<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApplicationSelectionIndicator {
    #[default]
    ByDefault,
    CustomerChoice,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelCard {
    number: cards::CardNumber,
    expiry: CardExpiryDate,
    security_code: Secret<String>,
    card_holder_name: Secret<String>,
    application_selection_indicator: ApplicationSelectionIndicator,
    scheme: ArchipelCardScheme,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelWallet {
    wallet_indicator: Secret<String>,
    wallet_provider: Secret<String>,
    wallet_cryptogram: Secret<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Eq, PartialEq)]
pub enum ThreeDsAuthStatus {
    Y,
    N,
    U,
    C,
    R,
    A,
    D,
    I,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Archipel3DS {
    #[serde(rename="acsTransID")]
    acs_trans_id: Option<Secret<String>>,
    #[serde(rename="dsTransID")]
    ds_trans_id: Option<Secret<String>>,
    #[serde(rename="3DSRequestorName")]
    three_ds_requestor_name: Option<Secret<String>>,
    #[serde(rename="3DSAuthDate")]
    three_ds_auth_date: Option<String>,
    #[serde(rename="3DSAuthAmt")]
    three_ds_auth_amt: Option<u32>,
    #[serde(rename="3DSAuthStatus")]
    three_ds_auth_status: Option<ThreeDsAuthStatus>,
    #[serde(rename="3DSMaxSupportedVersion")]
    three_ds_max_supported_version: Option<String>,
    #[serde(rename="3DSVersion")]
    three_ds_version: Option<String>,
    authentication_value: Option<Secret<String>>,
    authentication_method: Option<Secret<String>>,
    eci: Option<Secret<String>>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelCardHolder {
    billing_address: Option<ArchipelBillingAddress>,
}

impl TryFrom<Option<ArchipelBillingAddress>> for ArchipelCardHolder {
    type Error = ();
    fn try_from(value: Option<ArchipelBillingAddress>) -> Result<Self, Self::Error> {
        Ok(Self {
            billing_address: value
        })
    }
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelBillingAddress {
    address: Secret<String>,
    postal_code: Secret<String>,
}

impl TryFrom<api_models::payments::AddressDetails> for ArchipelBillingAddress {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(address_details: api_models::payments::AddressDetails) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address_details.get_combined_address_line()?,
            postal_code: address_details.get_zip()?.clone(),
        })
    }
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ArchipelCredentialIndicatorStatus {
    Initial,
    Subsequent
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelCredentialIndicator {
    status: ArchipelCredentialIndicatorStatus,
    recurring: Option<bool>,
    transaction_id: Option<String>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelAuthorizationRequest {
    order: ArchipelOrderRequest,
    card: Option<ArchipelCard>,
    cardholder: Option<ArchipelCardHolder>,
    wallet: Option<ArchipelWallet>,
    #[serde(rename="3DS")]
    three_ds: Option<Archipel3DS>,
    credential_indicator: Option<ArchipelCredentialIndicator>,
    stored_on_file: bool,
    tenant_id: String,
    token_id: Option<String>,
}

impl TryFrom<&ArchipelRouterData<&types::PaymentsAuthorizeRouterData>> for ArchipelAuthorizationRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &ArchipelRouterData<&types::PaymentsAuthorizeRouterData>) -> Result<Self,Self::Error> {
        let order = ArchipelOrderRequest {
            amount: item.amount.to_owned(),
            currency: item.router_data.request.currency.to_string(),
            certainty: ArchipelPaymentCertainty::Final,
            initiator: ArchipelPaymentInitiator::Customer
        };
        let billing_addr = item.router_data.get_billing()?.clone();
        let card_holder_name = billing_addr
            .get_optional_full_name()
            .ok_or(errors::ConnectorError::MissingRequiredField {
                field_name: "card.card_holder_name"
            })?;

        let payment_information = match item.router_data.request.payment_method_data.clone() {
            domain::PaymentMethodData::Card(ccard) => {
                ArchipelPaymentInformation::CardPayment (
                    CardPaymentInformation {
                        card: ArchipelCard {
                            number: ccard.card_number.clone(),
                            expiry: CardExpiryDate {
                                month: ccard.card_exp_month.clone(),
                                year: ccard.get_card_expiry_year_2_digit().unwrap().clone(),
                            },
                            security_code: ccard.card_cvc.clone(),
                            // TODO: Set with default value. Not yet implemented on HP
                            application_selection_indicator: ApplicationSelectionIndicator::ByDefault,
                            card_holder_name,
                            scheme: ArchipelCardScheme::foreign_from(ccard.get_card_issuer().ok())

                        },
                        wallet: None,
                        three_ds: None,
                    }
                )
            }
            // TODO: Implement wallet
            domain::PaymentMethodData::Wallet(_) |
            domain::PaymentMethodData::CardRedirect(_) |
            domain::PaymentMethodData::PayLater(_) |
            domain::PaymentMethodData::BankRedirect(_) |
            domain::PaymentMethodData::BankDebit(_) |
            domain::PaymentMethodData::BankTransfer(_) |
            domain::PaymentMethodData::Crypto(_) |
            domain::PaymentMethodData::MandatePayment |
            domain::PaymentMethodData::Reward |
            domain::PaymentMethodData::Upi(_) |
            domain::PaymentMethodData::Voucher(_) |
            domain::PaymentMethodData::GiftCard(_) |
            domain::PaymentMethodData::CardToken(_) |
            domain::PaymentMethodData::RealTimePayment(_) |
            domain::PaymentMethodData::OpenBanking(_) => {
                Err(errors::ConnectorError::NotImplemented(
                    utils::get_unimplemented_payment_method_error_message("Archipel"),
                ))?
            }
        };

        let (card, wallet, three_ds): (Option<ArchipelCard>, Option<ArchipelWallet>, Option<Archipel3DS>) = match payment_information {
            ArchipelPaymentInformation::CardPayment(cpay) => {
                (Some(cpay.card), cpay.wallet, cpay.three_ds)
            }
            ArchipelPaymentInformation::WalletPayment(wpay) => {
                (wpay.card, Some(wpay.wallet), Some(wpay.three_ds))
            }
        };

        let cardholder = Some(ArchipelCardHolder {
            billing_address: ArchipelBillingAddress::try_from(billing_addr.address
                .ok_or(errors::ConnectorError::MissingRequiredField {
                    field_name: "billing.address"
                })?
            ).ok()
        });

        // TODO: bind credentialsIndicator
        let credential_indicator = Some(ArchipelCredentialIndicator {
            status: ArchipelCredentialIndicatorStatus::Initial,
            recurring: Some(false),
            transaction_id: None
        });

        // TODO: bind stored_on_file. False by default
        let stored_on_file = false;

        let tenant_id: String = item.tenant_id.clone();


        // TODO: bind tenant_id
        let token_id: Option<String> = None;

        Ok(Self {
            order,
            cardholder,
            card,
            wallet,
            three_ds,
            credential_indicator,
            stored_on_file,
            tenant_id,
            token_id,
        })
    }
}

// PaymentsResponse

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ArchipelCardScheme {
    Amex,
    Mastercard,
    Visa,
    Discover,
    Diners,
    Unknown,
}

impl ForeignFrom<Option<CardIssuer>> for ArchipelCardScheme {
    fn foreign_from(card_issuer: Option<CardIssuer>) -> Self {
        if !card_issuer.is_none() {
            match card_issuer.unwrap() {
                CardIssuer::Visa => ArchipelCardScheme::Visa,
                CardIssuer::Master |
                CardIssuer::Maestro => ArchipelCardScheme::Mastercard,
                CardIssuer::AmericanExpress => ArchipelCardScheme::Amex,
                CardIssuer::Discover => ArchipelCardScheme::Discover,
                CardIssuer::DinersClub => ArchipelCardScheme::Diners,
                _ => ArchipelCardScheme::Unknown,
            }
        }
        else {
            ArchipelCardScheme::Unknown
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ArchipelPaymentStatus {
    Pending,
    Accepted,
    Refused,
    Error,
}
impl ForeignTryFrom<(enums::AttemptStatus, enums::CaptureMethod)> for ArchipelPaymentCase {
    type Error = errors::ConnectorError;

    fn foreign_try_from((status, capture_method):
                        (enums::AttemptStatus, enums::CaptureMethod)) -> Result<Self,Self::Error> {
        let is_auto_capture = match capture_method {
            enums::CaptureMethod::Automatic => { true },
            _ => { false }
        };

        match status {
            enums::AttemptStatus::AuthenticationFailed => {
                Ok(ArchipelPaymentCase::Verify)
            },
            enums::AttemptStatus::Authorizing |
            enums::AttemptStatus::Authorized |
            enums::AttemptStatus::AuthorizationFailed => {
                Ok(ArchipelPaymentCase::Authorize)
            },
            enums::AttemptStatus::Voided |
            enums::AttemptStatus::VoidInitiated |
            enums::AttemptStatus::VoidFailed => {
                Ok(ArchipelPaymentCase::Cancel)
            },
            enums::AttemptStatus::CaptureInitiated |
            enums::AttemptStatus::CaptureFailed => {
                if is_auto_capture {
                    Ok(ArchipelPaymentCase::Pay)
                } else {
                    Ok(ArchipelPaymentCase::Capture)
                }
            },
            enums::AttemptStatus::PaymentMethodAwaited |
            enums::AttemptStatus::ConfirmationAwaited => {
                if is_auto_capture {
                    Ok(ArchipelPaymentCase::Pay)
                } else {
                    Ok(ArchipelPaymentCase::Authorize)
                }
            },
            _ => {
                Err(errors::ConnectorError::ProcessingStepFailed(
                    Some(Bytes::from_static("Impossible to determine Archipel flow from AttemptStatus".to_bytes())))
                )
            }
        }
    }
}

// TODO: Add all possible cases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArchipelPaymentCase {
    Verify,
    Authorize,
    Pay,
    Capture,
    Cancel,
}

impl ForeignFrom<(ArchipelPaymentStatus, ArchipelPaymentCase)> for enums::AttemptStatus {
    fn foreign_from((status, archipel_flow): (ArchipelPaymentStatus, ArchipelPaymentCase)) -> Self {
        match status {
            ArchipelPaymentStatus::Accepted => {
                match archipel_flow {
                    ArchipelPaymentCase::Capture |
                    ArchipelPaymentCase::Pay |
                    ArchipelPaymentCase::Verify => {
                        Self::Charged
                    },
                    ArchipelPaymentCase::Cancel => {
                        Self::Voided
                    }
                    _ => {
                        Self::Authorized
                    }
                }
            }
            ArchipelPaymentStatus::Pending => {
                match archipel_flow {
                    ArchipelPaymentCase::Capture |
                    ArchipelPaymentCase::Pay => {
                        Self::CaptureInitiated
                    },
                    ArchipelPaymentCase::Cancel => {
                        Self::VoidInitiated
                    }
                    _ => {
                        Self::Authorizing
                    }
                }
            },
            ArchipelPaymentStatus::Refused => {
                match archipel_flow {
                    ArchipelPaymentCase::Capture => {
                        Self::CaptureFailed
                    },
                    ArchipelPaymentCase::Cancel => {
                        Self::VoidFailed
                    }
                    _ => {
                        Self::AuthorizationFailed
                    }
                }
            }
            ArchipelPaymentStatus::Error => {
                Self::Failure
            }
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchipelErrorResponse {
    pub status_code: u16,
    pub code: String,
    pub message: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelOrderResponse {
    id: String,
    amount: Option<i64>,
    currency: Option<enums::Currency>,
    captured_amount: Option<i64>,
    authorized_amount: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchipelErrorMessage {
    pub code: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelTransactionMetadata {
    pub transaction_id: String,
    pub transaction_date: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelTransactionReference {
    pub financial_network_code: Option<String>,
    pub issuer_transaction_id: Option<String>,
    pub response_code: Option<String>,
    pub authorization_code: Option<String>,
    pub payment_account_reference: Option<String>,
}

impl From<&ArchipelTransactionReference> for types::AdditionalPaymentMethodConnectorResponse {
    fn from(transaction_reference: &ArchipelTransactionReference) -> Self {
        let payment_checks = Some(
            serde_json::json!(transaction_reference),
        );

        Self::Card {
            authentication_data: None,
            payment_checks,
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelPaymentsResponse {
    order: ArchipelOrderResponse,
    transaction_id: String,
    transaction_date: String,
    status: ArchipelPaymentStatus,
    error: Option<ArchipelErrorMessage>,
    financial_network_code: Option<String>,
    issuer_transaction_id: Option<String>,
    response_code: Option<String>,
    authorization_code: Option<String>,
    payment_account_reference: Option<String>,
}

impl From<&ArchipelPaymentsResponse> for ArchipelTransactionMetadata {
    fn from(payment_response: &ArchipelPaymentsResponse) -> Self {
        Self {
            transaction_id: payment_response.transaction_id.clone(),
            transaction_date: payment_response.transaction_date.clone()
        }
    }
}

impl From<&ArchipelPaymentsResponse> for ArchipelTransactionReference {
    fn from(payment_response: &ArchipelPaymentsResponse) -> Self {
        Self {
            financial_network_code: payment_response.financial_network_code.clone(),
            issuer_transaction_id: payment_response.issuer_transaction_id.clone(),
            response_code: payment_response.response_code.clone(),
            authorization_code: payment_response.authorization_code.clone(),
            payment_account_reference: payment_response.payment_account_reference.clone()
        }
    }
}

// Handle responses for Payments Authorization Flow
impl<F> TryFrom<
    types::ResponseRouterData<F,
        ArchipelPaymentsResponse,
        types::PaymentsAuthorizeData,
        types::PaymentsResponseData>> for types::RouterData<F, types::PaymentsAuthorizeData, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::ResponseRouterData<F,
        ArchipelPaymentsResponse,
        types::PaymentsAuthorizeData,
        types::PaymentsResponseData>) -> Result<Self, Self::Error> {

        let capture_method = item.data.request.capture_method
            .clone()
            .ok_or(errors::ConnectorError::CaptureMethodNotSupported)?;

        let archipel_flow = if capture_method == enums::CaptureMethod::Automatic {
            ArchipelPaymentCase::Pay
        } else {
            ArchipelPaymentCase::Authorize
        };

        let status = enums::AttemptStatus::foreign_from(
            (item.response.status.clone(), archipel_flow)
        );

        let metadata: Option<serde_json::Value> = ArchipelTransactionMetadata::from(&item.response)
            .encode_to_value()
            .ok();

        let transaction_reference: Option<types::ConnectorResponseData> = Some(
            types::ConnectorResponseData::with_additional_payment_method_data(
                types::AdditionalPaymentMethodConnectorResponse::from(
                    &ArchipelTransactionReference::from(&item.response)
                )
            )
        );

        let is_incremental_allowed = if capture_method == enums::CaptureMethod::Automatic {
            false
        }
        else {
            item.data.request.request_incremental_authorization
        };

        Ok(Self {
            status,
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.order.id.clone()),
                charge_id: None,
                redirection_data: None,
                mandate_reference: None,
                connector_metadata: metadata,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: Some(is_incremental_allowed),
            }),
            connector_response: transaction_reference,
            ..item.data
        })
    }
}

/* PSync Flow */
impl<F> TryFrom<types::ResponseRouterData<F,
    ArchipelPaymentsResponse,
    types::PaymentsSyncData,
    types::PaymentsResponseData>> for types::RouterData<F, types::PaymentsSyncData, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::ResponseRouterData<
        F,
        ArchipelPaymentsResponse,
        types::PaymentsSyncData,
        types::PaymentsResponseData>) -> Result<Self,Self::Error> {

        let capture_method = item.data.request.capture_method
            .clone()
            .ok_or(errors::ConnectorError::CaptureMethodNotSupported)?;

        let archipel_flow = ArchipelPaymentCase::foreign_try_from(
            (item.data.status.clone(), capture_method)
        )?;

        let status = enums::AttemptStatus::foreign_from(
            (item.response.status.clone(), archipel_flow)
        );

        let metadata: Option<serde_json::Value> = ArchipelTransactionMetadata::from(&item.response)
            .encode_to_value()
            .ok();

        let payment_checks: Option<types::ConnectorResponseData> = Some(
            types::ConnectorResponseData::with_additional_payment_method_data(
                types::AdditionalPaymentMethodConnectorResponse::from(
                    &ArchipelTransactionReference::from(&item.response)
                )
            )
        );

        Ok(Self {
            status,
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.order.id.to_owned()),
                charge_id: None,
                redirection_data: None,
                mandate_reference: None,
                connector_metadata: metadata,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: None,
            }),
            connector_response: payment_checks,
            amount_captured: item.response.order.captured_amount.to_owned(),
            ..item.data
        })
    }
}

/* CAPTURE FLOW */

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct ArchipelCaptureRequest {
    order: ArchipelCaptureOrderRequest,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct ArchipelCaptureOrderRequest {
    amount: i64,
}

impl TryFrom<&ArchipelRouterData<&types::PaymentsCaptureRouterData>> for ArchipelCaptureRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &ArchipelRouterData<&types::PaymentsCaptureRouterData>) -> Result<Self,Self::Error> {
        Ok(Self {
            order: ArchipelCaptureOrderRequest {
                amount: item.amount.to_owned(),
            }
        })
    }
}

impl<F> TryFrom<types::ResponseRouterData<F,
    ArchipelPaymentsResponse,
    types::PaymentsCaptureData,
    types::PaymentsResponseData>> for types::RouterData<F, types::PaymentsCaptureData, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::ResponseRouterData<
        F,
        ArchipelPaymentsResponse,
        types::PaymentsCaptureData,
        types::PaymentsResponseData>) -> Result<Self,Self::Error> {

        let status = enums::AttemptStatus::foreign_from((item.response.status.clone(), ArchipelPaymentCase::Capture));

        let connector_metadata: Option<serde_json::Value> = ArchipelTransactionMetadata::from(&item.response)
            .encode_to_value()
            .ok();

        let payment_checks: Option<types::ConnectorResponseData> = Some(
            types::ConnectorResponseData::with_additional_payment_method_data(
                types::AdditionalPaymentMethodConnectorResponse::from(
                    &ArchipelTransactionReference::from(&item.response)
                )
            )
        );

        Ok(Self {
            status,
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.order.id.to_owned()),
                charge_id: None,
                redirection_data: None,
                mandate_reference: None,
                connector_metadata,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: None,
            }),
            connector_response: payment_checks,
            ..item.data
        })
    }
}

/* REFUND FLOW */
//TODO: Fill the struct with respective fields
// Type definition for RefundRequest
#[derive(Default, Debug, Serialize)]
pub struct ArchipelRefundRequest {
    pub amount: i64
}

impl<F> TryFrom<&ArchipelRouterData<&types::RefundsRouterData<F>>> for ArchipelRefundRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &ArchipelRouterData<&types::RefundsRouterData<F>>) -> Result<Self,Self::Error> {
        Ok(Self {
            amount: item.amount.to_owned(),
        })
    }
}

// Type definition for Refund Response

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum RefundStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Succeeded => Self::Success,
            RefundStatus::Failed => Self::Failure,
            RefundStatus::Processing => Self::Pending,
            //TODO: Review mapping
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    id: String,
    status: RefundStatus
}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, RefundResponse>>
for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::RefundsResponseRouterData<api::Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.status),
            }),
            ..item.data
        })
    }
}

impl TryFrom<types::RefundsResponseRouterData<api::RSync, RefundResponse>> for types::RefundsRouterData<api::RSync>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::RefundsResponseRouterData<api::RSync, RefundResponse>) -> Result<Self,Self::Error> {
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.status),
            }),
            ..item.data
        })
    }
}

// Setup Mandate FLow

impl TryFrom<&ArchipelRouterData<&types::SetupMandateRouterData>> for ArchipelAuthorizationRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &ArchipelRouterData<&types::SetupMandateRouterData>) -> Result<Self,Self::Error> {
        let order = ArchipelOrderRequest {
            amount: item.amount.to_owned(),
            currency: item.router_data.request.currency.to_string(),
            certainty: ArchipelPaymentCertainty::Final,
            initiator: ArchipelPaymentInitiator::Customer
        };
        let billing_addr = item.router_data.get_billing()?.clone();
        let card_holder_name = billing_addr
            .get_optional_full_name()
            .ok_or(errors::ConnectorError::MissingRequiredField {
                field_name: "card.card_holder_name"
            })?;

        let payment_information = match item.router_data.request.payment_method_data.clone() {
            domain::PaymentMethodData::Card(ccard) => {
                ArchipelPaymentInformation::CardPayment (
                    CardPaymentInformation {
                        card: ArchipelCard {
                            number: ccard.card_number.clone(),
                            expiry: CardExpiryDate {
                                month: ccard.card_exp_month.clone(),
                                year: ccard.get_card_expiry_year_2_digit().unwrap().clone(),
                            },
                            security_code: ccard.card_cvc.clone(),
                            // TODO: Set with default value. Not yet implemented on HP
                            application_selection_indicator: ApplicationSelectionIndicator::ByDefault,
                            card_holder_name,
                            scheme: ArchipelCardScheme::foreign_from(ccard.get_card_issuer().ok())
                        },
                        wallet: None,
                        three_ds: None,
                    }
                )
            }
            // TODO: Implement wallet
            domain::PaymentMethodData::Wallet(_) |
            domain::PaymentMethodData::CardRedirect(_) |
            domain::PaymentMethodData::PayLater(_) |
            domain::PaymentMethodData::BankRedirect(_) |
            domain::PaymentMethodData::BankDebit(_) |
            domain::PaymentMethodData::BankTransfer(_) |
            domain::PaymentMethodData::Crypto(_) |
            domain::PaymentMethodData::MandatePayment |
            domain::PaymentMethodData::Reward |
            domain::PaymentMethodData::Upi(_) |
            domain::PaymentMethodData::Voucher(_) |
            domain::PaymentMethodData::GiftCard(_) |
            domain::PaymentMethodData::CardToken(_) |
            domain::PaymentMethodData::RealTimePayment(_) |
            domain::PaymentMethodData::OpenBanking(_) => {
                Err(errors::ConnectorError::NotImplemented(
                    utils::get_unimplemented_payment_method_error_message("Archipel"),
                ))?
            }
        };

        let (card, wallet, three_ds): (Option<ArchipelCard>, Option<ArchipelWallet>, Option<Archipel3DS>) = match payment_information {
            ArchipelPaymentInformation::CardPayment(cpay) => {
                (Some(cpay.card), cpay.wallet, cpay.three_ds)
            }
            ArchipelPaymentInformation::WalletPayment(wpay) => {
                (wpay.card, Some(wpay.wallet), Some(wpay.three_ds))
            }
        };

        let cardholder = Some(ArchipelCardHolder {
            billing_address: ArchipelBillingAddress::try_from(billing_addr.address
                .ok_or(errors::ConnectorError::MissingRequiredField {
                    field_name: "billing.address"
                })?
            ).ok()
        });

        let credential_indicator = Some(ArchipelCredentialIndicator {
            status: ArchipelCredentialIndicatorStatus::Initial,
            recurring: Some(false),
            transaction_id: None
        });

        let tenant_id: String = item.tenant_id.clone();

        Ok(Self {
            order,
            cardholder,
            card,
            wallet,
            three_ds,
            credential_indicator,
            stored_on_file: true,
            tenant_id,
            token_id: None,
        })
    }
}

impl<F> TryFrom<types::ResponseRouterData<F,
    ArchipelPaymentsResponse,
    types::SetupMandateRequestData,
    types::PaymentsResponseData>> for types::RouterData<F, types::SetupMandateRequestData, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::ResponseRouterData<
        F,
        ArchipelPaymentsResponse,
        types::SetupMandateRequestData,
        types::PaymentsResponseData>) -> Result<Self,Self::Error> {

        let status = enums::AttemptStatus::foreign_from(
            (item.response.status.clone(), ArchipelPaymentCase::Verify)
        );

        let metadata: Option<serde_json::Value> = ArchipelTransactionMetadata::from(&item.response)
            .encode_to_value()
            .ok();

        let payment_checks: Option<types::ConnectorResponseData> = Some(
            types::ConnectorResponseData::with_additional_payment_method_data(
                types::AdditionalPaymentMethodConnectorResponse::from(
                    &ArchipelTransactionReference::from(&item.response)
                )
            )
        );

        Ok(Self {
            status,
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.order.id.to_owned()),
                charge_id: None,
                redirection_data: None,
                mandate_reference: None,
                connector_metadata: metadata,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: None,
            }),
            connector_response: payment_checks,
            ..item.data
        })
    }
}

//      Void Flow => /cancel/{order_id}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelPaymentsCancelRequest {
    tenant_id: String
}

impl TryFrom<&ArchipelRouterData<&types::PaymentsCancelRouterData>> for ArchipelPaymentsCancelRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &ArchipelRouterData<&types::PaymentsCancelRouterData>) -> Result<Self,Self::Error> {
        let tenant_id: String = item.tenant_id.clone();
        Ok(Self { tenant_id })
    }
}

impl<F> TryFrom<types::ResponseRouterData<F,
    ArchipelPaymentsResponse,
    types::PaymentsCancelData,
    types::PaymentsResponseData>
> for types::RouterData<F, types::PaymentsCancelData, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::ResponseRouterData<
        F,
        ArchipelPaymentsResponse,
        types::PaymentsCancelData,
        types::PaymentsResponseData>
    ) -> Result<Self,Self::Error> {

        let status = enums::AttemptStatus::foreign_from(
            (item.response.status.clone(), ArchipelPaymentCase::Cancel)
        );

        let metadata: Option<serde_json::Value> = ArchipelTransactionMetadata::from(&item.response)
            .encode_to_value()
            .ok();

        let payment_checks: Option<types::ConnectorResponseData> = Some(
            types::ConnectorResponseData::with_additional_payment_method_data(
                types::AdditionalPaymentMethodConnectorResponse::from(
                    &ArchipelTransactionReference::from(&item.response)
                )
            )
        );

        Ok(Self {
            status,
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.order.id.to_owned()),
                charge_id: None,
                redirection_data: None,
                mandate_reference: None,
                connector_metadata: metadata,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: None,
            }),
            connector_response: payment_checks,
            ..item.data
        })
    }
}





#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelIncrementalAuthorizationRequest {
    order: ArchipelOrderRequest,
    tenant_id: String,
}

// Incremental Authorization status mapping
impl From<ArchipelPaymentStatus> for enums::AuthorizationStatus {
    fn from(status: ArchipelPaymentStatus) -> Self {
        match status {
            ArchipelPaymentStatus::Accepted => Self::Success,
            ArchipelPaymentStatus::Pending => Self::Processing,
            ArchipelPaymentStatus::Error |
            ArchipelPaymentStatus::Refused => Self::Failure,
        }
    }
}

impl TryFrom<&ArchipelRouterData<
    &types::PaymentsIncrementalAuthorizationRouterData>
> for ArchipelIncrementalAuthorizationRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &ArchipelRouterData<&types::PaymentsIncrementalAuthorizationRouterData>)
        -> Result<Self,Self::Error> {
        Ok(Self {
            order: ArchipelOrderRequest {
                amount: item.amount.to_owned(),
                currency: item.router_data.request.currency.to_string(),
                certainty: ArchipelPaymentCertainty::Estimated,
                initiator: ArchipelPaymentInitiator::Customer,
            },
            tenant_id: item.tenant_id.clone()
        })
    }
}

impl<F> TryFrom<types::ResponseRouterData<F,
    ArchipelPaymentsResponse,
    types::PaymentsIncrementalAuthorizationData,
    types::PaymentsResponseData>> for types::RouterData<F, types::PaymentsIncrementalAuthorizationData, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::ResponseRouterData<
        F,
        ArchipelPaymentsResponse,
        types::PaymentsIncrementalAuthorizationData,
        types::PaymentsResponseData>) -> Result<Self,Self::Error> {

        let status = enums::AuthorizationStatus::from(item.response.status.clone());

        let connector_response: Option<types::ConnectorResponseData> = Some(
            types::ConnectorResponseData::with_additional_payment_method_data(
                types::AdditionalPaymentMethodConnectorResponse::from(
                    &ArchipelTransactionReference::from(&item.response)
                )
            )
        );

        let errors: (Option<String>, Option<String>) =
            if status.clone() == enums::AuthorizationStatus::Success ||
                item.response.error.clone().is_none() { (None, None) }
            else {
                let archipel_error = item.response.error.clone().unwrap();
                (Some(archipel_error.code), archipel_error.description)
            };

        Ok(Self {
            response: Ok(types::PaymentsResponseData::IncrementalAuthorizationResponse {
                status,
                error_code: errors.0,
                error_message: errors.1,
                connector_authorization_id: None
            }),
            connector_response,
            ..item.data
        })
    }
}