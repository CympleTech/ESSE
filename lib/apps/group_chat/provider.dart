import 'package:flutter/material.dart';

import 'package:esse/rpc.dart';
import 'package:esse/apps/group_chat/models.dart';

class GroupChatProvider extends ChangeNotifier {
  Map<int, int> groups = {};

  GroupChatProvider() {
    // rpc.
    rpc.addListener('group-chat-list', _list, false);
    // rpc.addListener('group-chat-online', _online, false);
    // rpc.addListener('group-chat-offline', _online, false);
    rpc.addListener('group-chat-check', _check, false);
    rpc.addListener('group-chat-create', _create, false);
    // rpc.addListener('group-chat-update', _update, false);
    // rpc.addListener('group-chat-join', _join, true);
    // rpc.addListener('group-chat-agree', _agree, true);
    // rpc.addListener('group-chat-reject', _reject, false);
    // rpc.addListener('group-chat-member-join', _memberJoin, false);
    // rpc.addListener('group-chat-member-info', _memberInfo, false);
    // rpc.addListener('group-chat-member-leave', _memberLeave, false);
    // rpc.addListener('group-chat-member-online', _memberOnline, false);
    // rpc.addListener('group-chat-member-offline', _memberOffline, false);
    // rpc.addListener('group-chat-message-create', _messageCreate, true);
    // rpc.addListener('group-chat-message-delete', _messageDelete, false);
    // rpc.addListener('group-chat-message-delivery', _messageDelivery, false);
  }

  clear() {
  }

  updateActived() {
    this.clear();

    // load groups.
    rpc.send('group-chat-list', []);
  }

  check(String addr) {
    rpc.send('group-chat-check', [addr]);
  }

  create() {
    rpc.send('group-chat-create', []);
  }

  _list(List params) {
    this.clear();
    params.forEach((params) {
        // if (params.length == 6) {
        //   this.devices[params[0]] = Device.fromList(params);
        // }
    });
    notifyListeners();
  }

  _check(List params) {
    //
  }

  _create(List params) {
    //
  }
}
