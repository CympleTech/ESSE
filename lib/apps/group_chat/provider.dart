import "dart:collection";

import 'package:flutter/material.dart';

import 'package:esse/rpc.dart';
import 'package:esse/apps/group_chat/models.dart';
import 'package:esse/apps/chat/models.dart' show Message;

class GroupChatProvider extends ChangeNotifier {
  List<GroupType> createSupported = [GroupType.Encrypted, GroupType.Common, GroupType.Open];
  CheckType createCheckType = CheckType.Wait;

  Map<int, GroupChat> groups = {};
  List<int> createKeys = [];
  List<int> orderKeys = [];
  SplayTreeMap<int, GroupChat> requests = SplayTreeMap();

  int actived;
  SplayTreeMap<int, Message> activedMessages = SplayTreeMap();

  GroupChat get activedGroup => this.groups[this.actived];

  GroupChatProvider() {
    // rpc.
    rpc.addListener('group-chat-list', _list, false);
    // rpc.addListener('group-chat-online', _online, false);
    // rpc.addListener('group-chat-offline', _online, false);
    rpc.addListener('group-chat-check', _check, false);
    rpc.addListener('group-chat-create', _create, false);
    rpc.addListener('group-chat-result', _result, false);
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

  updateActivedGroup(int id) {
    this.actived = id;
    // TODO load
  }

  clearActivedGroup() {
    // TODO
  }

  check(String addr) {
    rpc.send('group-chat-check', [addr]);
  }

  create(String addr, String name, String bio, bool needAgree) {
    rpc.send('group-chat-create', [addr, name, bio, needAgree]);
  }

  reSend(int id) {
    //
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
    this.createSupported.clear();
    this.createCheckType = CheckTypeExtension.fromInt(params[0]);
    params[1].forEach((param) {
        this.createSupported.add(GroupTypeExtension.fromInt(param));
    });
    notifyListeners();
  }

  _create(List params) {
    final gc = GroupChat.fromList(params);
    if (gc.isOk) {
      this.orderKeys.add(gc.id);
    } else {
      this.createKeys.add(gc.id);
    }
    this.groups[gc.id] = gc;

    notifyListeners();
  }

  _result(List params) {
    final id = params[0];
    this.groups[id].isOk = params[1];
    this.groups[id].online = true;
    if (params[1]) {
      //this.createKeys.remove(id);
      this.orderKeys.add(id);
    }
    notifyListeners();
  }
}
