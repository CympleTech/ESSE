import "dart:collection";
import 'package:flutter/material.dart';

import 'package:esse/rpc.dart';
import 'package:esse/apps/assistant/models.dart';

class AssistantProvider extends ChangeNotifier {
  bool isActived = false;
  SplayTreeMap<int, Message> messages = SplayTreeMap();

  AssistantProvider() {
    // rpc.
    rpc.addListener('assistant-list', _list, false);
    rpc.addListener('assistant-create', _create, false);
    rpc.addListener('assistant-update', _update, false);
    rpc.addListener('assistant-delete', _delete, false);
  }

  actived() {
    this.isActived = true;
    rpc.send('assistant-list', []);
  }

  inactived() {
    this.messages.clear();
    this.isActived = false;
  }

  create(MessageType q_type, String q_content) {
    rpc.send('assistant-create', [q_type.toInt(), q_content]);
    notifyListeners();
  }

  /// delete a message.
  delete(int id) {
    this.messages.remove(id);
    rpc.send('assistant-delete', [id]);
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
        this.messages[id].update(params);
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
