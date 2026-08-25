load(js.exec_dir + "ct-alerts-lib.js");

var db;
try {
	db = CTAlerts.open_database();
	var linked = CTAlerts.session(db);
	CTAlerts.json({
		ok: true,
		vapidPublicKey: CTAlerts.setting(db, "vapid_public_key"),
		linked: Boolean(linked),
		preferences: linked ? CTAlerts.preferences(db, linked.id) : null
	});
	db.close();
} catch (error) {
	if (db)
		db.close();
	CTAlerts.handle_error(error);
}
