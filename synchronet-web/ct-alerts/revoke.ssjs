load(js.exec_dir + "ct-alerts-lib.js");

var db;
try {
	db = CTAlerts.open_database();
	CTAlerts.require_post(db);
	var linked = CTAlerts.require_session(db);
	var now = time();
	db.transaction(function () {
		db.run("UPDATE subscriptions SET revoked_unix=? WHERE session_id=? AND revoked_unix IS NULL",
		    [now, linked.id]);
		db.run("UPDATE browser_sessions SET revoked_unix=? WHERE id=? AND revoked_unix IS NULL",
		    [now, linked.id]);
	});
	var public_url = CTAlerts.setting(db, "public_url");
	http_reply.header["Set-Cookie"] = "ct_device=; Path=" +
	    CTAlerts.path(public_url) +
	    "; Max-Age=0; Secure; HttpOnly; SameSite=Strict";
	CTAlerts.json({ ok: true, linked: false });
	db.close();
} catch (error) {
	if (db)
		db.close();
	CTAlerts.handle_error(error);
}
