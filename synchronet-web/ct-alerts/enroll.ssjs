load(js.exec_dir + "ct-alerts-lib.js");

var db;
try {
	db = CTAlerts.open_database();
	var request = CTAlerts.require_post(db);
	var token = CTAlerts.require_string(request, "token", 22, 128,
	    /^[A-Za-z0-9_-]+$/);
	var credential = CTAlerts.require_string(request, "deviceCredential", 43, 43,
	    /^[A-Za-z0-9_-]+$/);
	var endpoint = CTAlerts.require_string(request, "endpoint", 12, 4096,
	    /^https:\/\//i);
	var p256dh = CTAlerts.require_string(request, "p256dh", 80, 160,
	    /^[A-Za-z0-9_-]+$/);
	var auth = CTAlerts.require_string(request, "auth", 20, 64,
	    /^[A-Za-z0-9_-]+$/);
	var locale = typeof request.locale === "string" &&
	    /^[A-Za-z0-9-]{2,20}$/.test(request.locale) ? request.locale : "en-US";
	var preferences = request.preferences || {};
	var lead = Number(preferences.leadMinutes);
	if (!isFinite(lead) || Math.floor(lead) !== lead || lead < 1 || lead > 1440)
		lead = 5;
	var now = time();
	var linked_session_id = null;

	db.transaction(function () {
		var tokens = db.query(
		    "SELECT bbs_id,player_id FROM pairing_tokens " +
		    "WHERE token_hash=? AND consumed_unix IS NULL AND expires_unix>=?",
		    [sha256_calc(token, true), now]);
		if (tokens.length !== 1)
			CTAlerts.fail("410 Gone", "This pairing signal has expired or was already used.");
		var bbs_id = Number(tokens[0].bbs_id);
		var player_id = Number(tokens[0].player_id);
		var existing = db.query(
		    "SELECT s.id,s.session_id,s.bbs_id,s.player_id FROM subscriptions s " +
		    "WHERE s.endpoint=? AND s.revoked_unix IS NULL", [endpoint]);
		if (existing.length &&
		    (Number(existing[0].bbs_id) !== bbs_id ||
		     Number(existing[0].player_id) !== player_id))
			CTAlerts.fail("409 Conflict", "This receiver is already linked to another captain.");

		if (existing.length) {
			linked_session_id = Number(existing[0].session_id);
			db.run("UPDATE browser_sessions SET credential_hash=? WHERE id=?",
			    [sha256_calc(credential, true), linked_session_id]);
		} else {
			var counts = db.query(
			    "SELECT COUNT(DISTINCT session_id) AS count FROM subscriptions " +
			    "WHERE bbs_id=? AND player_id=? AND revoked_unix IS NULL",
			    [bbs_id, player_id]);
			if (Number(counts[0].count) >= 5)
				CTAlerts.fail("409 Conflict", "Five receivers are already linked.");
			db.run(
			    "INSERT INTO browser_sessions(credential_hash,bbs_id,player_id,created_unix) " +
			    "VALUES(?,?,?,?)",
			    [sha256_calc(credential, true), bbs_id, player_id, now]);
			var sessions = db.query(
			    "SELECT id FROM browser_sessions WHERE credential_hash=?",
			    [sha256_calc(credential, true)]);
			linked_session_id = Number(sessions[0].id);
		}

		db.run(
		    "INSERT INTO subscriptions(session_id,bbs_id,player_id,endpoint,p256dh,auth," +
		    "locale,attention_soon,attention_now,automation_applied,lead_minutes," +
		    "created_unix,updated_unix,revoked_unix,failure_count) " +
		    "VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,NULL,0) " +
		    "ON CONFLICT(endpoint) DO UPDATE SET session_id=excluded.session_id," +
		    "bbs_id=excluded.bbs_id,player_id=excluded.player_id,p256dh=excluded.p256dh," +
		    "auth=excluded.auth,locale=excluded.locale,attention_soon=excluded.attention_soon," +
		    "attention_now=excluded.attention_now,automation_applied=excluded.automation_applied," +
		    "lead_minutes=excluded.lead_minutes,updated_unix=excluded.updated_unix," +
		    "revoked_unix=NULL,failure_count=0",
		    [linked_session_id, bbs_id, player_id, endpoint, p256dh, auth, locale,
		     CTAlerts.boolean_value(preferences, "attentionSoon", true) ? 1 : 0,
		     CTAlerts.boolean_value(preferences, "attentionNow", true) ? 1 : 0,
		     CTAlerts.boolean_value(preferences, "automationApplied", true) ? 1 : 0,
		     lead, now, now]);
		db.run(
		    "UPDATE pairing_tokens SET consumed_unix=? " +
		    "WHERE token_hash=? AND consumed_unix IS NULL",
		    [now, sha256_calc(token, true)]);
	});

	var public_url = CTAlerts.setting(db, "public_url");
	http_reply.header["Set-Cookie"] = "ct_device=" + credential +
	    "; Path=" + CTAlerts.path(public_url) +
	    "; Max-Age=31536000; Secure; HttpOnly; SameSite=Strict";
	CTAlerts.json({
		ok: true,
		linked: true,
		preferences: CTAlerts.preferences(db, linked_session_id)
	});
	db.close();
} catch (error) {
	if (db)
		db.close();
	CTAlerts.handle_error(error);
}
