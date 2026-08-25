(function () {
  "use strict";

  var state = { config: null, token: null, notice: null };
  var byId = function (id) { return document.getElementById(id); };

  function setStatus(title, text, signal) {
    byId("status-title").textContent = title;
    byId("status-text").textContent = text;
    byId("signal").textContent = signal || "STANDBY";
  }

  function base64url(buffer) {
    var bytes = new Uint8Array(buffer);
    var binary = "";
    for (var i = 0; i < bytes.length; i++)
      binary += String.fromCharCode(bytes[i]);
    return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  }

  function vapidBytes(value) {
    var padded = value.replace(/-/g, "+").replace(/_/g, "/");
    while (padded.length % 4)
      padded += "=";
    var raw = atob(padded);
    var bytes = new Uint8Array(raw.length);
    for (var i = 0; i < raw.length; i++)
      bytes[i] = raw.charCodeAt(i);
    return bytes;
  }

  function post(path, value) {
    return fetch(path, {
      method: "POST",
      credentials: "same-origin",
      cache: "no-store",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(value)
    }).then(function (response) {
      return response.json().catch(function () {
        throw new Error("The relay returned an unreadable response.");
      }).then(function (body) {
        if (!response.ok || !body.ok)
          throw new Error(body.error || "The relay rejected the transmission.");
        return body;
      });
    });
  }

  function preferenceObject() {
    return {
      attentionSoon: byId("attention-soon").checked,
      attentionNow: byId("attention-now").checked,
      automationApplied: true,
      leadMinutes: Number(byId("lead-minutes").value)
    };
  }

  function showPreferences(preferences) {
    preferences = preferences || {
      attentionSoon: true,
      attentionNow: true,
      automationApplied: true,
      leadMinutes: 5
    };
    byId("attention-soon").checked = preferences.attentionSoon;
    byId("attention-now").checked = preferences.attentionNow;
    byId("lead-minutes").value = preferences.leadMinutes;
    byId("settings").hidden = false;
    byId("enrollment").hidden = true;
  }

  function setBusy(button, busy) {
    button.disabled = busy;
    byId("signal").textContent = busy ? "TRANSMITTING" : "LINKED";
  }

  function linkDevice() {
    var button = byId("link-button");
    setBusy(button, true);
    if (!state.token) {
      setBusy(button, false);
      setStatus("Pairing signal missing", "Open a new link from the game's universal menu.", "NO SIGNAL");
      return;
    }
    if (!("serviceWorker" in navigator) || !("PushManager" in window)) {
      setBusy(button, false);
      setStatus("Receiver unsupported", "This browser cannot receive Web Push messages.", "INCOMPATIBLE");
      return;
    }

    Notification.requestPermission().then(function (permission) {
      if (permission !== "granted")
        throw new Error("Notification permission was not granted.");
      return navigator.serviceWorker.register("service-worker.js");
    }).then(function (registration) {
      return registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: vapidBytes(state.config.vapidPublicKey)
      });
    }).then(function (subscription) {
      var json = subscription.toJSON();
      var random = new Uint8Array(32);
      crypto.getRandomValues(random);
      return post("enroll.ssjs", {
        token: state.token,
        deviceCredential: base64url(random),
        endpoint: subscription.endpoint,
        p256dh: json.keys.p256dh,
        auth: json.keys.auth,
        locale: navigator.language || "en-US",
        preferences: preferenceObject()
      });
    }).then(function (response) {
      history.replaceState(null, "", location.pathname);
      state.token = null;
      state.config.linked = true;
      showPreferences(response.preferences);
      setStatus("Receiver linked", "Bridge calls will reach this communicator according to the orders below.", "LINKED");
    }).catch(function (error) {
      setStatus("Link failed", error.message, "FAULT");
    }).then(function () {
      setBusy(button, false);
    });
  }

  function savePreferences() {
    var button = byId("save-button");
    setBusy(button, true);
    post("preferences.ssjs", preferenceObject()).then(function (response) {
      showPreferences(response.preferences);
      setStatus("Orders acknowledged", "The bridge relay has updated this receiver.", "LINKED");
    }).catch(function (error) {
      setStatus("Orders rejected", error.message, "FAULT");
    }).then(function () {
      setBusy(button, false);
    });
  }

  function revokeDevice() {
    if (!window.confirm("Unlink this communicator from your captain?"))
      return;
    var button = byId("revoke-button");
    setBusy(button, true);
    post("revoke.ssjs", {}).then(function () {
      return navigator.serviceWorker.ready;
    }).then(function (registration) {
      return registration.pushManager.getSubscription();
    }).then(function (subscription) {
      return subscription ? subscription.unsubscribe() : true;
    }).then(function () {
      byId("settings").hidden = true;
      setStatus("Receiver unlinked", "This unit will no longer receive bridge calls. Link it again from the game when needed.", "STANDBY");
    }).catch(function (error) {
      setStatus("Unlink failed", error.message, "FAULT");
    }).then(function () {
      setBusy(button, false);
    });
  }

  function detailLabel(value) {
    return value.replace(/([A-Z])/g, " $1").replace(/_/g, " ")
      .replace(/^./, function (letter) { return letter.toUpperCase(); });
  }

  function appendDetails(list, value, prefix) {
    if (!value || typeof value !== "object")
      return;
    Object.keys(value).forEach(function (key) {
      var item = value[key];
      var name = prefix ? prefix + " · " + detailLabel(key) : detailLabel(key);
      if (item !== null && typeof item === "object") {
        appendDetails(list, item, name);
      } else {
        var term = document.createElement("dt");
        var description = document.createElement("dd");
        term.textContent = name;
        description.textContent = String(item);
        list.appendChild(term);
        list.appendChild(description);
      }
    });
  }

  function loadNotice() {
    post("notice.ssjs", { notice: state.notice }).then(function (response) {
      byId("notice-title").textContent = response.alert.title;
      byId("notice-body").textContent = response.alert.body;
      var list = byId("notice-detail");
      list.textContent = "";
      appendDetails(list, response.alert.detail, "");
      byId("notice-panel").hidden = false;
      byId("status-panel").hidden = true;
      byId("signal").textContent = "PRIORITY";
      history.replaceState(null, "", location.pathname);
    }).catch(function (error) {
      setStatus("Transmission unavailable", error.message, "EXPIRED");
    });
  }

  function parseFragment() {
    var fragment = location.hash.substring(1);
    if (!fragment)
      return;
    if (fragment.indexOf("notice=") === 0)
      state.notice = decodeURIComponent(fragment.substring(7));
    else
      state.token = fragment;
  }

  function start() {
    parseFragment();
    byId("link-button").addEventListener("click", linkDevice);
    byId("save-button").addEventListener("click", savePreferences);
    byId("revoke-button").addEventListener("click", revokeDevice);
    fetch("config.ssjs", { credentials: "same-origin", cache: "no-store" })
      .then(function (response) { return response.json(); })
      .then(function (config) {
        if (!config.ok)
          throw new Error(config.error || "The bridge relay is unavailable.");
        state.config = config;
        if (state.notice) {
          loadNotice();
        } else if (config.linked) {
          if (state.token)
            history.replaceState(null, "", location.pathname);
          showPreferences(config.preferences);
          setStatus("Receiver linked", "This communicator is standing by for bridge calls.", "LINKED");
        } else if (state.token) {
          byId("enrollment").hidden = false;
          setStatus("Pairing signal acquired", "Permission is required before this unit can receive bridge calls.", "SIGNAL LOCK");
        } else {
          setStatus("Awaiting pairing signal", "In Cepheus Trader, open the universal menu and choose Browser Alerts.", "STANDBY");
        }
      }).catch(function (error) {
        setStatus("Relay unavailable", error.message, "OFFLINE");
      });
  }

  start();
})();
