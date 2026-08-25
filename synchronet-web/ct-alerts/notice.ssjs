load(js.exec_dir + "ct-alerts-lib.js");

var db;
try {
	db = CTAlerts.open_database();
	var request = CTAlerts.require_post(db);
	var linked = CTAlerts.require_session(db);
	var notice = CTAlerts.require_string(request, "notice", 22, 64,
	    /^[A-Za-z0-9_-]+$/);
	var rows = db.query(
	    "SELECT kind,title,body,detail_json,created_unix,expires_unix FROM alerts " +
	    "WHERE notice_ref=? AND bbs_id=? AND player_id=? AND expires_unix>?",
	    [notice, linked.bbs_id, linked.player_id, time()]);
	if (rows.length !== 1)
		CTAlerts.fail("404 Not Found", "This transmission is unavailable or has expired.");
	var detail;
	try {
		detail = JSON.parse(String(rows[0].detail_json));
	} catch (error) {
		detail = { message: String(rows[0].detail_json) };
	}
	CTAlerts.json({
		ok: true,
		alert: {
			kind: String(rows[0].kind),
			title: String(rows[0].title),
			body: String(rows[0].body),
			detail: detail,
			createdUnix: Number(rows[0].created_unix),
			expiresUnix: Number(rows[0].expires_unix)
		}
	});
	db.close();
} catch (error) {
	if (db)
		db.close();
	CTAlerts.handle_error(error);
}
