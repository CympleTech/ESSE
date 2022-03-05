import 'package:flutter/material.dart';

import 'package:esse/rpc.dart';
import 'package:esse/apps/device/models.dart';

class DeviceProvider extends ChangeNotifier {
  Map<int, Device> devices = {};
  DeviceStatus status = DeviceStatus();

  DeviceProvider() {
    // rpc.
    rpc.addListener('device-list', _list);
    rpc.addListener('device-create', _create);
    rpc.addListener('device-delete', _delete);
    rpc.addListener('device-online', _online);
    rpc.addListener('device-offline', _offline);
    rpc.addListener('device-status', _status);
  }

  clear() {
    this.status = DeviceStatus();
  }

  updateActived() {
    this.clear();

    // load devices.
    rpc.send('device-list', []);
  }

  updateActivedDevice(int id) {
    this.clear();

    // load status.
    rpc.send('device-status', [id]);
  }

  connect(String addr) {
    rpc.send('device-connect', [addr]);
  }

  delete(int id) {
    this.devices.remove(id);
    rpc.send('device-delete', [id]);
    notifyListeners();
  }

  _list(List params) {
    this.devices.clear();
    params.forEach((params) {
        if (params.length == 6) {
          this.devices[params[0]] = Device.fromList(params);
        }
    });
    notifyListeners();
  }

  _create(List params) {
    if (params.length == 6) {
      this.devices[params[0]] = Device.fromList(params);
      notifyListeners();
    }
  }

  _delete(List params) {
    final id = params[0];
    if (this.devices.containsKey(id)) {
      this.devices.remove(id);
      notifyListeners();
    }
  }

  _online(List params) {
    final id = params[0];
    if (this.devices.containsKey(id)) {
      this.devices[id]!.online = true;
      notifyListeners();
    }
  }

  _offline(List params) {
    final id = params[0];
    if (this.devices.containsKey(id)) {
      this.devices[id]!.online = false;
      notifyListeners();
    }
  }

  _status(List params) {
    if (params.length == 9) {
      this.status = DeviceStatus.fromList(params);
      notifyListeners();
    }
  }
}
