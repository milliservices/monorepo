wrk.method = "POST"
wrk.body = "Hello world testing some body herej"
wrk.headers["Content-Type"] = "application/x-www-form-urlencoded"
wrk.headers["authentication"] = "wrk-benchmark"
