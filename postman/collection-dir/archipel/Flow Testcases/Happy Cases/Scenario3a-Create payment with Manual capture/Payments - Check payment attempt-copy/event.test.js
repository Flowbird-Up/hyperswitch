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
  "[POST]::/payments - Payment Attempt has 'charged' status",
  function () {
    pm.expect(payment_attempt.status).to.be.equal("charged");
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
  "[POST]::/payments - Payment Attempt 'transactionId' connector_metadata fiels",
  function () {
    meta = pm.collectionVariables.get("archipel_transaction_meta")
    // transactionId must have changed, not null and not empty
    pm.expect(payment_attempt.connector_metadata.transactionId).to.not.null
    pm.expect(payment_attempt.connector_metadata.transactionId).to.not.equal("");
    pm.expect(payment_attempt.connector_metadata.transactionId).to.not.equal(meta.transactionId);
  },
);

pm.test(
  "[POST]::/payments - Payment Attempt match additional connector_metadata fiels",
  function () {
    meta = pm.collectionVariables.get("archipel_transaction_meta")
    pm.expect(payment_attempt.connector_metadata.responseCode).to.equal(meta.responseCode);
    pm.expect(payment_attempt.connector_metadata.authorizationCode).to.equal(meta.authorizationCode);
    pm.expect(payment_attempt.connector_metadata.transactionDate).to.equal(meta.transactionDate);
    pm.expect(payment_attempt.connector_metadata.issuerTransactionId).to.equal(meta.issuerTransactionId);
    pm.expect(payment_attempt.connector_metadata.financialNetworkCode).to.equal(meta.financialNetworkCode);
    pm.expect(payment_attempt.connector_metadata.paymentAccountReference).to.equal(meta.paymentAccountReference);
  },
);