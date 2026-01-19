const { Client, HttpMethod } = require("./index");

async function test() {
    const client = new Client({
        impersonate: "chrome_120",
        impersonate_os: "windows",
        verify: false,
    });

    try {
        const response = await client.request(
            HttpMethod.GET,
            "https://tls.peet.ws/api/all",
        );

        console.log("Response Status:", response.statusCode);
        console.log("Response Headers:", response.headers());

        const jsonBody = await response.json();

        if (jsonBody.tls) {
            console.log("TLS Fingerprint Summary:");
            console.log("  - JA3:", jsonBody.tls.ja3);
            console.log("  - JA4:", jsonBody.tls.ja4);
        }

        if (jsonBody.http && jsonBody.http.headers) {
            console.log("User-Agent:", jsonBody.http.headers["user-agent"]);
        } else {
            console.log("headers info not present in response.");
        }
    } catch (error) {
        console.error("failed", error);
    }
}

test();
