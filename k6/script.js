import http from "k6/http";
import { check } from "k6";

export let options = {
    stages: [
        // Ramp-up from 1 to 30 VUs in 30s
        { duration: "30s", target: 30 },

        // Stay on 30 VUs for 60s
        { duration: "60s", target: 30 },

        // Ramp-down from 30 to 0 VUs in 10s
        { duration: "10s", target: 0 }
    ]
};

export default function() {
    let res = http.get("http://web:8080/health");
    check(res, { "status is 200": (r) => r.status === 200 });
}