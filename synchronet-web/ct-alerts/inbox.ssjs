load(js.exec_dir + "ct-alerts-lib.js");

var db;
try {
	db = CTAlerts.open_database();
	CTAlerts.require_post(db);
	var linked = CTAlerts.require_session(db);
	var rows = db.query(
	    "SELECT DISTINCT a.notice_ref,a.kind,a.title,a.body," +
	    "a.created_unix,a.expires_unix FROM alerts a " +
	    "JOIN deliveries d ON d.alert_id=a.id " +
	    "JOIN subscriptions s ON s.id=d.subscription_id " +
	    "WHERE s.session_id=? " +
	    "AND a.bbs_id=? AND a.player_id=? " +
	    "AND d.state='delivered' AND a.expires_unix>? " +
	    "ORDER BY a.created_unix DESC",
	    [linked.id, linked.bbs_id, linked.player_id, time()]);
	var alerts = [];
	for (var i = 0; i < rows.length; i++) {
		alerts.push({
			notice: String(rows[i].notice_ref),
			kind: String(rows[i].kind),
			title: String(rows[i].title),
			body: String(rows[i].body),
			createdUnix: Number(rows[i].created_unix),
			expiresUnix: Number(rows[i].expires_unix)
		});
	}
	CTAlerts.json({ ok: true, alerts: alerts });
	db.close();
} catch (error) {
	if (db)
		db.close();
	CTAlerts.handle_error(error);
}
