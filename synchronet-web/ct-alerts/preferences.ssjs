load(js.exec_dir + "ct-alerts-lib.js");

var db;
try {
	db = CTAlerts.open_database();
	var request = CTAlerts.require_post(db);
	var linked = CTAlerts.require_session(db);
	var lead = Number(request.leadMinutes);
	if (!isFinite(lead) || Math.floor(lead) !== lead || lead < 1 || lead > 1440)
		CTAlerts.fail("400 Bad Request", "Lead time must be from 1 to 1440 minutes.");
	var result = db.run(
	    "UPDATE subscriptions SET attention_soon=?,attention_now=?,automation_applied=?," +
	    "lead_minutes=?,updated_unix=? WHERE session_id=? AND revoked_unix IS NULL",
	    [CTAlerts.boolean_value(request, "attentionSoon", true) ? 1 : 0,
	     CTAlerts.boolean_value(request, "attentionNow", true) ? 1 : 0,
	     CTAlerts.boolean_value(request, "automationApplied", true) ? 1 : 0,
	     lead, time(), linked.id]);
	CTAlerts.json({
		ok: true,
		preferences: CTAlerts.preferences(db, linked.id)
	});
	db.close();
} catch (error) {
	if (db)
		db.close();
	CTAlerts.handle_error(error);
}
