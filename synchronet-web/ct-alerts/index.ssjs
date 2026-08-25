load(js.exec_dir + "ct-alerts-lib.js");
CTAlerts.security_headers("text/html; charset=utf-8");
http_reply.header["Content-Security-Policy"] =
    "default-src 'self'; script-src 'self'; style-src 'self'; " +
    "img-src 'self' data:; connect-src 'self'; object-src 'none'; " +
    "base-uri 'none'; form-action 'none'; frame-ancestors 'none'; " +
    "worker-src 'self'; manifest-src 'self'";

load(js.exec_dir + "communicator-page.js");
write(CT_COMMUNICATOR_PAGE);
