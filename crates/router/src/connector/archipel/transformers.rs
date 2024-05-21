use serde::{Deserialize, Serialize};
use api_models::payments::AddressDetails;
use masking::{Secret};
use crate::{core::errors, types::{self, api, storage::enums}};
use crate::connector::utils;
use crate::connector::utils::{AddressDetailsData, RouterData};
use crate::types::domain;

//TODO: Fill the struct with respective fields
pub struct ArchipelRouterData<T> {
    pub amount: i64, // The type of amount that a connector accepts, for example, String, i64, f64, etc.
    pub router_data: T,
}

impl<T>
TryFrom<(
    &types::api::CurrencyUnit,
    types::storage::enums::Currency,
    i64,
    T,
)> for ArchipelRouterData<T>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        (_currency_unit, _currency, amount, item): (
            &types::api::CurrencyUnit,
            types::storage::enums::Currency,
            i64,
            T,
        ),
    ) -> Result<Self, Self::Error> {
        //Todo :  use utils to convert the amount to the type of amount that a connector accepts
        Ok(Self {
            amount,
            router_data: item,
        })
    }
}

#[derive(Default, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelOrder {
    amount: i64,
    currency: i64,
    wallet_indicator: String,
    wallet_provider: String,
}

#[derive(Default, Debug, Serialize, Eq, PartialEq)]
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

#[derive(Default, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelCard {
    number: cards::CardNumber,
    expiry: CardExpiryDate,
    security_code: Secret<String>,
    card_holder_name: Option<Secret<String>>,
    application_selection_indicator: ApplicationSelectionIndicator,
    scheme: Option<String>,
}

#[derive(Default, Debug, Serialize, Eq, PartialEq)]
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

#[derive(Default, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelBillingAddress {
    address: Option<Secret<String>>,
    postal_code: Option<Secret<String>>,
}

impl TryFrom<Option<AddressDetails>> for ArchipelBillingAddress {
    type Error = ();
    fn try_from(_address_details: Option<AddressDetails>) -> Result<Self, Self::Error> {
        let details = _address_details.unwrap();
        Ok(Self {
            address: details.get_combined_address_line().ok(),
            postal_code: details.zip,
        })
    }
}

#[derive(Default, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelCredentialIndicator {
    status: Option<String>,
    recurring: Option<bool>,
    transaction_id: Option<String>,
}

#[derive(Default, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelPaymentsRequest {
    order: ArchipelOrder,
    card: ArchipelCard,
    cardholder: Option<ArchipelCardHolder>,
    #[serde(rename="3DS")]
    three_ds: Option<String>,
    credential_indicator: Option<ArchipelCredentialIndicator>,
    stored_on_file: bool,
    tenant_id: u32,
    token_id: Option<String>,
}

impl TryFrom<&ArchipelRouterData<&types::PaymentsAuthorizeRouterData>> for ArchipelPaymentsRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &ArchipelRouterData<&types::PaymentsAuthorizeRouterData>) -> Result<Self,Self::Error> {
        let order = ArchipelOrder {
            amount: item.amount.to_owned(),
            currency: item.router_data.request.currency.iso_4217().parse::<i64>().unwrap(),
            // TODO: implement wallet info
            wallet_indicator: "".to_string(),
            wallet_provider: "".to_string(),
        };


        let card_data = match item.router_data.request.payment_method_data.clone() {
           domain::PaymentMethodData::Card(ccard) => {
               ccard
           }
           | domain::PaymentMethodData::Wallet(_)
           | domain::PaymentMethodData::CardRedirect(_)
           | domain::PaymentMethodData::PayLater(_)
           | domain::PaymentMethodData::BankRedirect(_)
           | domain::PaymentMethodData::BankDebit(_)
           | domain::PaymentMethodData::BankTransfer(_)
           | domain::PaymentMethodData::Crypto(_)
           | domain::PaymentMethodData::MandatePayment
           | domain::PaymentMethodData::Reward
           | domain::PaymentMethodData::Upi(_)
           | domain::PaymentMethodData::Voucher(_)
           | domain::PaymentMethodData::GiftCard(_)
           | domain::PaymentMethodData::CardToken(_) => {
               Err(errors::ConnectorError::NotImplemented(
                   utils::get_unimplemented_payment_method_error_message("Archipel"),
               ))?
           }
       };
        let card = ArchipelCard {
               number: card_data.card_number,
               expiry: CardExpiryDate {
                   month: card_data.card_exp_month,
                   year: card_data.card_exp_year,
               },
               security_code: card_data.card_cvc,
               application_selection_indicator: ApplicationSelectionIndicator::ByDefault,
               card_holder_name: card_data.nick_name,
               scheme: card_data.card_issuer
           };

        let billing_details = item.router_data.get_billing()?.clone().address.clone().or(None);
        let cardholder = Some(ArchipelCardHolder {
            billing_address: ArchipelBillingAddress::try_from(billing_details).ok()
        });

        // TODO: bind credentialsIndicator
        let credential_indicator = Some(ArchipelCredentialIndicator {
            status: None,
            recurring: None,
            transaction_id: None
        });

        Ok(Self {
            order,
            card,
            three_ds: None,
            cardholder,
            credential_indicator,
            stored_on_file: false,
            // TODO: Bind tenantId
            tenant_id: 0,
            token_id: None
        })
    }
}

//TODO: Fill the struct with respective fields
// Auth Struct
pub struct ArchipelAuthType {
    pub(super) api_key: Secret<String>,
}

impl TryFrom<&types::ConnectorAuthType> for ArchipelAuthType  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(auth_type: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
        match auth_type {
            types::ConnectorAuthType::HeaderKey { api_key } => Ok(Self {
                api_key: api_key.to_owned(),
            }),
            _ => Err(errors::ConnectorError::FailedToObtainAuthType.into()),
        }
    }
}
// PaymentsResponse
//TODO: Append the remaining status flags
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ArchipelPaymentStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<ArchipelPaymentStatus> for enums::AttemptStatus {
    fn from(item: ArchipelPaymentStatus) -> Self {
        match item {
            ArchipelPaymentStatus::Succeeded => Self::Charged,
            ArchipelPaymentStatus::Failed => Self::Failure,
            ArchipelPaymentStatus::Processing => Self::Authorizing,
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchipelPaymentsResponse {
    status: ArchipelPaymentStatus,
    id: String,
}

impl<F,T> TryFrom<types::ResponseRouterData<F, ArchipelPaymentsResponse, T, types::PaymentsResponseData>> for types::RouterData<F, T, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::ResponseRouterData<F, ArchipelPaymentsResponse, T, types::PaymentsResponseData>) -> Result<Self,Self::Error> {
        Ok(Self {
            status: enums::AttemptStatus::from(item.response.status),
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.id),
                redirection_data: None,
                mandate_reference: None,
                connector_metadata: None,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: None,
            }),
            ..item.data
        })
    }
}

//TODO: Fill the struct with respective fields
// REFUND :
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

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ArchipelErrorResponse {
    pub status_code: u16,
    pub code: String,
    pub message: String,
    pub reason: Option<String>,
}
