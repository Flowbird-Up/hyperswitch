import * as fixtures from "../../fixtures/imports";
import State from "../../utils/State";
import { payment_methods_enabled } from "../PaymentUtils/Commons";

let globalState;
let connector;
describe("Connector Account Create flow test", () => {
  before("seed global state", () => {
    cy.task("getGlobalState").then((state) => {
      globalState = new State(state);
      connector = globalState.get("connectorId");
    });
  });

  after("flush global state", () => {
    cy.task("setGlobalState", globalState.data);
  });

  it("connector-create-call-test", () => {
    cy.createConnectorCallTest(
      "payment_processor",
      fixtures.createConnectorBody,
      payment_methods_enabled,
      globalState
    );
  });

  it("Enable Connector Agnostic for Business Profile", () => {
    if (connector === "archipel") {
      cy.UpdateBusinessProfileTest(
          fixtures.businessProfile.bpUpdate,
          true, // is_connector_agnostic_enabled
          false, // collect_billing_address_from_wallet_connector
          false, // collect_shipping_address_from_wallet_connector
          false, // always_collect_billing_address_from_wallet_connector
          false, // always_collect_shipping_address_from_wallet_connector
          globalState
      );
    } else {
      cy.log(
          `Connector Agnostic not enabled for ${connector}. Skipping Business Profile update`
      );
    }
  });

  it("check and create multiple connectors", () => {
    const multiple_connectors = Cypress.env("MULTIPLE_CONNECTORS");
    // multiple_connectors will be undefined if not set in the env
    if (multiple_connectors?.status) {
      // Create multiple connectors based on the count
      // The first connector is already created when creating merchant account, so start from 1
      for (let i = 1; i < multiple_connectors.count; i++) {
        cy.createBusinessProfileTest(
          fixtures.businessProfile.bpCreate,
          globalState,
          "profile" + i
        );
        cy.createConnectorCallTest(
          "payment_processor",
          fixtures.createConnectorBody,
          payment_methods_enabled,
          globalState,
          `profile${i}`,
          `merchantConnector${i}`
        );
      }
    } else {
      cy.log(
        "Multiple connectors not enabled. Skipping creation of multiple profiles and respective MCAs"
      );
    }
  });
});
