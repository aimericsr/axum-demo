import http from "k6/http";
import { check, group, sleep } from "k6";

// You first need to add the cert to you OS store as k6 don't provide this options for self-signed certs
// k6 run infrastructure/docker-compose/k6/script.js
export const options = {
  stages: [
    { duration: "15s", target: 300 },
    { duration: "30s", target: 600 },
    { duration: "10s", target: 0 },
  ],
  thresholds: {
    http_req_duration: ["p(95)<100", "p(90)<80"],
    http_req_failed: ["rate<0.01"],
  },
  tlsAuth: [
    {
      cert: open(`${__ENV.HOME}/ssl/my-app.crt`),
      key: open(`${__ENV.HOME}/ssl/my-app.key`),
    },
  ],
};

// k6 run infrastructure/docker-compose/k6/script.js
// Default user worflow
export default function () {
  const BASE_URL = "https://my-app";

  group("typical user workflow", function () {
    group("health check general", function () {
      const res = http.get(`${BASE_URL}/health`);
      check(res, {
        "status is 200": (res) => res.status === 200,
      });
    });

    group("health check ready", function () {
      const res = http.get(`${BASE_URL}/health/ready`);
      check(res, {
        "status is 200": (res) => res.status === 200,
      });
    });
  });

  sleep(0.3);

  group("login", function () {});
}
