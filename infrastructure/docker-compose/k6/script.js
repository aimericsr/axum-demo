import http from "k6/http";
import { check, group, sleep } from "k6";

export const options = {
    stages: [
        { duration: "30s", target: 30 },
        { duration: "60s", target: 30 },
        { duration: "10s", target: 0 }
    ],
    thresholds: {
        http_req_duration: ["p(95)<100", "p(90)<80"],
        http_req_failed: ['rate<0.01'],
    }
};

export default function() {
    const BASE_URL = 'http://web:8080'

    group('health check', function () {
        group("health check live", function() {
            const res = http.get(`${BASE_URL}/health/live`);
            check(res, {
                "status is 200": (res) => res.status === 200,
            });
        });
    
        group("health check ready", function() {
            const res = http.get(`${BASE_URL}/health/ready`);
            check(res, {
                "status is 200": (res) => res.status === 200,
            });
        });
        sleep(0.3)
    });

    group('login', function () {
        
    });
}