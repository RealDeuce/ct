/* Shared server-side helpers for the Cepheus Trader browser-alert site. */

var CTAlerts = (function () {
	function fail(status, message) {
		throw { ct_status: status, message: message };
	}

	function header(name) {
		var wanted = name.toLowerCase();
		for (var key in http_request.header) {
			if (key.toLowerCase() === wanted)
				return String(http_request.header[key]);
		}
		return "";
	}

	function read_config() {
		var file = new File(file_cfgname(system.ctrl_dir,
		    "cepheus-trader-web-push.ini"));
		if (!file.open("r"))
			fail("503 Service Unavailable",
			    "The communicator relay has not been configured.");
		var config = file.iniGetObject() || {};
		file.close();
		if (typeof config.database !== "string" || !config.database.length)
			fail("503 Service Unavailable",
			    "The communicator database is not configured.");
		return config;
	}

	function open_database() {
		var config = read_config();
		var db = new SQLite(config.database);
		db.exec("PRAGMA foreign_keys = ON");
		var versions = db.query("SELECT version FROM schema_meta");
		if (versions.length !== 1 || Number(versions[0].version) !== 1) {
			db.close();
			fail("503 Service Unavailable",
			    "The communicator database requires service attention.");
		}
		return db;
	}

	function setting(db, name) {
		var rows = db.query("SELECT value FROM settings WHERE name=?", [name]);
		if (rows.length !== 1)
			fail("503 Service Unavailable", "A relay setting is missing.");
		return String(rows[0].value);
	}

	function origin(public_url) {
		var match = /^(https:\/\/[^\/]+)/i.exec(public_url);
		if (!match)
			fail("503 Service Unavailable", "The relay address is invalid.");
		return match[1].toLowerCase().replace(/:443$/, "");
	}

	function path(public_url) {
		var match = /^https:\/\/[^\/]+(\/[^?#]*)/i.exec(public_url);
		var value = match ? match[1] : "/";
		if (value.charAt(value.length - 1) !== "/")
			value = value.substring(0, value.lastIndexOf("/") + 1);
		return value || "/";
	}

	function require_post(db) {
		if (String(http_request.method).toUpperCase() !== "POST")
			fail("405 Method Not Allowed", "This circuit accepts POST only.");
		var public_url = setting(db, "public_url");
		if (header("Origin").toLowerCase() !== origin(public_url))
			fail("403 Forbidden", "The transmission origin was rejected.");
		if (!http_request.post_data || http_request.post_data.length > 8192)
			fail("400 Bad Request", "The transmission is empty or too large.");
		try {
			return JSON.parse(http_request.post_data);
		} catch (error) {
			fail("400 Bad Request", "The transmission is not valid JSON.");
		}
	}

	function require_string(object, name, minimum, maximum, pattern) {
		var value = object && object[name];
		if (typeof value !== "string" || value.length < minimum ||
		    value.length > maximum || (pattern && !pattern.test(value)))
			fail("400 Bad Request", "Invalid " + name + ".");
		return value;
	}

	function boolean_value(object, name, fallback) {
		return typeof object[name] === "boolean" ? object[name] : fallback;
	}

	function cookie_value(name) {
		var values = http_request.cookie[name];
		if (typeof values === "undefined")
			return null;
		if (!(values instanceof Array))
			values = [values];
		for (var i = 0; i < values.length; i++) {
			var value = String(values[i]);
			if (/^[A-Za-z0-9_-]{43}$/.test(value))
				return value;
		}
		return null;
	}

	function session(db) {
		var credential = cookie_value("ct_device");
		if (!credential)
			return null;
		var rows = db.query(
		    "SELECT id,bbs_id,player_id FROM browser_sessions " +
		    "WHERE credential_hash=? AND revoked_unix IS NULL",
		    [sha256_calc(credential, true)]);
		return rows.length === 1 ? rows[0] : null;
	}

	function require_session(db) {
		var value = session(db);
		if (!value)
			fail("401 Unauthorized", "This communicator is not linked.");
		return value;
	}

	function preferences(db, session_id) {
		var rows = db.query(
		    "SELECT attention_soon,attention_now,automation_applied,lead_minutes " +
		    "FROM subscriptions WHERE session_id=? AND revoked_unix IS NULL",
		    [session_id]);
		if (rows.length !== 1)
			return null;
		return {
			attentionSoon: Boolean(Number(rows[0].attention_soon)),
			attentionNow: Boolean(Number(rows[0].attention_now)),
			automationApplied: Boolean(Number(rows[0].automation_applied)),
			leadMinutes: Number(rows[0].lead_minutes)
		};
	}

	function security_headers(content_type) {
		http_reply.header["Content-Type"] = content_type;
		http_reply.header["Cache-Control"] = "no-store";
		http_reply.header["Pragma"] = "no-cache";
		http_reply.header["X-Content-Type-Options"] = "nosniff";
		http_reply.header["Referrer-Policy"] = "no-referrer";
		http_reply.header["X-Frame-Options"] = "DENY";
		http_reply.header["Permissions-Policy"] =
		    "camera=(), microphone=(), geolocation=(), payment=()";
	}

	function json(value, status) {
		security_headers("application/json; charset=utf-8");
		if (status)
			http_reply.status = status;
		write(JSON.stringify(value));
	}

	function handle_error(error) {
		var status = error && error.ct_status ? error.ct_status :
		    "500 Internal Server Error";
		var message = error && error.ct_status ? error.message :
		    "The communicator relay encountered an internal fault.";
		if (!error || !error.ct_status)
			log(LOG_ERR, "Cepheus Trader browser alerts: " + error);
		json({ ok: false, error: message }, status);
	}

	return {
		boolean_value: boolean_value,
		fail: fail,
		handle_error: handle_error,
		json: json,
		open_database: open_database,
		path: path,
		preferences: preferences,
		require_post: require_post,
		require_session: require_session,
		require_string: require_string,
		security_headers: security_headers,
		session: session,
		setting: setting
	};
})();
