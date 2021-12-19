import "dart:collection";
import 'package:flutter/material.dart';

import 'package:esse/rpc.dart';
import 'package:esse/apps/jarvis/models.dart';

class JarvisProvider extends ChangeNotifier {
  bool isActived = false;
  SplayTreeMap<int, Message> messages = SplayTreeMap();

  JarvisProvider() {
    // rpc.
    rpc.addListener('jarvis-list', _list, false);
    rpc.addListener('jarvis-create', _create, false);
    rpc.addListener('jarvis-update', _update, false);
    rpc.addListener('jarvis-delete', _delete, false);
  }

  actived() {
    this.isActived = true;
    rpc.send('jarvis-list', []);
  }

  inactived() {
    this.messages.clear();
    this.isActived = false;
  }

  create(MessageType qType, String qContent) {
    rpc.send('jarvis-create', [qType.toInt(), qContent]);
  }

  /// delete a message.
  delete(int id) {
    this.messages.remove(id);
    rpc.send('jarvis-delete', [id]);
    notifyListeners();
  }

  /// list message with friend.
  _list(List params) {
    if (this.isActived) {
      params.forEach((param) {
          this.messages[param[0]] = Message.fromList(param);
      });
      notifyListeners();
    }
  }

  /// friend send message to me.
  _create(List params) {
    if (this.isActived) {
      final msg = Message.fromList(params);
      this.messages[msg.id] = msg;
      notifyListeners();
    }
  }

  _update(List params) {
    if (this.isActived) {
      final id = params[0];
      if (this.messages.containsKey(id)) {
        this.messages[id]!.update(params);
        notifyListeners();
      }
    }
  }

  _delete(List params) {
    if (this.isActived) {
      this.messages.remove(params[0]);
      notifyListeners();
    }
  }
}
