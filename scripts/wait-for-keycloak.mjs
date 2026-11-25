const issuer = "http://localhost:8080/realms/rust-web-realm";
const discoveryUrl = `${issuer}/.well-known/openid-configuration`;
const deadline = Date.now() + 90_000;
let lastError;

while (Date.now() < deadline) {
  try {
    const response = await fetch(discoveryUrl, {
      signal: AbortSignal.timeout(2_000),
    });

    if (response.ok) {
      const discovery = await response.json();
      if (discovery.issuer === issuer) {
        console.log("Keycloak realm is ready");
        process.exit(0);
      }
      lastError = new Error(`Unexpected Keycloak issuer: ${discovery.issuer}`);
    } else {
      lastError = new Error(`Keycloak returned HTTP ${response.status}`);
    }
  } catch (error) {
    lastError = error;
  }

  await new Promise((resolve) => setTimeout(resolve, 1_000));
}

console.error("Keycloak did not become ready within 90 seconds", lastError);
process.exit(1);
