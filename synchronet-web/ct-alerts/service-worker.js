"use strict";

self.addEventListener("push", function (event) {
  var payload = {
    title: "Captain to the bridge!",
    body: "Cepheus Trader requires your attention.",
    tag: "ct-attention",
    notice: ""
  };
  if (event.data) {
    try {
      var incoming = event.data.json();
      if (typeof incoming.title === "string") payload.title = incoming.title;
      if (typeof incoming.body === "string") payload.body = incoming.body;
      if (typeof incoming.tag === "string") payload.tag = incoming.tag;
      if (typeof incoming.notice === "string") payload.notice = incoming.notice;
    } catch (error) {
      payload.body = event.data.text();
    }
  }
  event.waitUntil(self.registration.showNotification(payload.title, {
    body: payload.body,
    tag: payload.tag,
    renotify: true,
    requireInteraction: true,
    data: { notice: payload.notice }
  }));
});

self.addEventListener("notificationclick", function (event) {
  event.notification.close();
  var notice = event.notification.data && event.notification.data.notice;
  var target = "./" + (notice ? "#notice=" + encodeURIComponent(notice) : "");
  event.waitUntil(clients.matchAll({ type: "window", includeUncontrolled: true })
    .then(function (windows) {
      for (var i = 0; i < windows.length; i++) {
        if ("focus" in windows[i]) {
          windows[i].navigate(target);
          return windows[i].focus();
        }
      }
      return clients.openWindow ? clients.openWindow(target) : null;
    }));
});
