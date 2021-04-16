import 'package:flutter/material.dart';

import 'package:esse/models/account.dart';
import 'package:esse/models/device.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';

class DeviceProvider extends ChangeNotifier {
  Map<int, Device> devices = {};
  int activedId = -1;
  DeviceStatus status = DeviceStatus();

  init() {
    // rpc.
    rpc.addListener('device-list', _list);
    rpc.addListener('device-create', _create);
    rpc.addListener('device-delete', _delete);
    rpc.addListener('device-online', _online);
    rpc.addListener('device-offline', _offline);
    rpc.addListener('device-status', _status);

    // init.
    rpc.send('device-list', []);
  }

  updateActived(int id) {
    this.status = DeviceStatus();
    this.activedId = id;
    rpc.send('device-status', [this.devices[id].addr]);
  }

  clearActived() {
    this.activedId = -1;
  }

  connect(String addr) {
    rpc.send('device-connect', [addr]);
  }

  delete(int id) {
    this.activedId = -1;
    this.devices.remove(id);
    rpc.send('device-delete', [id]);
    notifyListeners();
  }

  _list(String gid, List params) {
    if (Global.gid == gid) {
      this.devices.clear();
      params.forEach((params) {
          if (params.length == 6) {
            this.devices[params[0]] = Device.fromList(params);
          }
      });
      notifyListeners();
    }
  }

  _create(String gid, List params) {
    if (Global.gid == gid) {
      if (params.length == 6) {
        this.devices[params[0]] = Device.fromList(params);
        notifyListeners();
      }
    }
  }

  _delete(String gid, List params) {
    if (Global.gid == gid) {
      this.devices.remove(params[0]);
      notifyListeners();
    }
  }

  _online(String gid, List params) {
    if (Global.gid == gid) {
      this.devices[params[0]].online = true;
      notifyListeners();
    }
  }

  _offline(String gid, List params) {
    if (Global.gid == gid) {
      this.devices[params[0]].online = false;
      notifyListeners();
    }
  }

  _status(String gid, List params) {
    if (Global.gid == gid) {
      if (params.length == 9) {
        this.status = DeviceStatus.fromList(params);
        notifyListeners();
      }
    }
  }
}
