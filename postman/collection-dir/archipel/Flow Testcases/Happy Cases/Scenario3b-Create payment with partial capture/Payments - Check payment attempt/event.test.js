// Validate status 2xx
pm.test("[GET]::/payments/:id - Status code is 2xx", function () {
  pm.response.to.be.success;
});

// Validate if response header has matching content-type
pm.test("[GET]::/payments/:id - Content-Type is application/json", function () {
  pm.expect(pm.response.headers.get("Content-Type")).to.include(
    "application/json",
  );
});

// Validate if response has JSON Body
pm.test("[GET]::/payments/:id - Response has JSON Body", function () {
  pm.response.to.have.jsonBody();
});

// Set response object as internal variable
let jsonData = {};
try {
  jsonData = pm.response.json();
} catch (e) {}


let payment_attempt = {}

pm.test(
  "[POST]::/payments - Payment has one 'Payment Attempt'",
  function () {
    pm.expect(jsonData.attempts.length).to.be.equal(1);
    payment_attempt = jsonData.attempts[0]
  },
);

pm.test(
  "[POST]::/payments - Payment Attempt has 'authorized' status",
  function () {
    pm.expect(payment_attempt.status).to.be.equal("authorized");
  },
);


pm.test(
  "[POST]::/payments - Payment Attempt has 'connector_transaction_id'",
  function () {
    pm.expect(jsonData.connector_transaction_id).to.be.equal(
        pm.collectionVariables.get("connector_transaction_id")
    );
  },
);


pm.test(
  "[POST]::/payments - Payment Attempt has no error",
  function () {
    pm.expect(payment_attempt.error_message).to.be.null;
  },
);

pm.test(
  "[POST]::/payments - Payment Attempt has connector_metadata",
  function () {
    pm.expect(payment_attempt.connector_metadata).to.not.be.null;
  },
);

pm.test(
  "[POST]::/payments - Payment Attempt has additional connector_metadata fiels",
  function () {
    pm.expect(payment_attempt.connector_metadata.responseCode).to.be.string;
    pm.expect(payment_attempt.connector_metadata.transactionId).to.be.string;
    pm.expect(payment_attempt.connector_metadata.transactionDate).to.be.string;
    pm.expect(payment_attempt.connector_metadata.authorizationCode).to.be.string;
    pm.expect(payment_attempt.connector_metadata.issuerTransactionId).to.be.string;
    pm.expect(payment_attempt.connector_metadata.financialNetworkCode).to.be.string;
    pm.expect(payment_attempt.connector_metadata.paymentAccountReference).to.be.string;
    pm.collectionVariables.set("archipel_transaction_meta", payment_attempt.connector_metadata);
  },
);