import "dart:collection";

import 'package:flutter/material.dart';

import 'package:esse/rpc.dart';
import 'package:esse/global.dart';

import 'package:esse/apps/primitives.dart';
import 'package:esse/apps/group_chat/models.dart';

class GroupChatProvider extends ChangeNotifier {
  List<GroupType> createSupported = [GroupType.Encrypted, GroupType.Common, GroupType.Open];
  CheckType createCheckType = CheckType.Wait;

  Map<int, GroupChat> groups = {};
  List<int> createKeys = [];
  List<int> orderKeys = [];
  SplayTreeMap<int, GroupChat> requests = SplayTreeMap();

  int actived;
  SplayTreeMap<int, Message> activedMessages = SplayTreeMap();
  SplayTreeMap<int, Member> activedMembers = SplayTreeMap();

  GroupChat get activedGroup => this.groups[this.actived];
  bool get isActivedGroupOwner => this.activedGroup.owner == Global.gid;
  bool get isActivedGroupManager {
    this.activedMembers.values.forEach((m) {
        if (m.mid == Global.gid) {
          return m.isManager;
        }
    });
    return false;
  }

  GroupChatProvider() {
    // rpc.
    rpc.addListener('group-chat-list', _list, false);
    rpc.addListener('group-chat-online', _online, false);
    rpc.addListener('group-chat-offline', _offline, false);
    rpc.addListener('group-chat-check', _check, false);
    rpc.addListener('group-chat-create', _create, false);
    rpc.addListener('group-chat-result', _result, false);
    rpc.addListener('group-chat-detail', _detail, true);
    // rpc.addListener('group-chat-update', _update, false);
    // rpc.addListener('group-chat-join', _join, true);
    // rpc.addListener('group-chat-agree', _agree, true);
    // rpc.addListener('group-chat-reject', _reject, false);
    rpc.addListener('group-chat-member-join', _memberJoin, false);
    // rpc.addListener('group-chat-member-info', _memberInfo, false);
    // rpc.addListener('group-chat-member-leave', _memberLeave, false);
    rpc.addListener('group-chat-member-online', _memberOnline, false);
    rpc.addListener('group-chat-member-offline', _memberOffline, false);
    rpc.addListener('group-chat-message-create', _messageCreate, true);
    // rpc.addListener('group-chat-message-delete', _messageDelete, false);
    // rpc.addListener('group-chat-message-delivery', _messageDelivery, false);
  }

  clear() {
    this.groups.clear();
    this.createKeys.clear();
    this.orderKeys.clear();
    this.requests.clear();
    this.activedMessages.clear();
    this.activedMembers.clear();
  }

  updateActived() {
    this.clear();

    // load groups.
    rpc.send('group-chat-list', []);
  }

  updateActivedGroup(int id) {
    this.actived = id;
    rpc.send('group-chat-detail', [id]);
  }

  clearActivedGroup() {
    this.activedMessages.clear();
    this.activedMembers.clear();
  }

  check(String addr) {
    rpc.send('group-chat-check', [addr]);
  }

  create(String myName, String addr, String name, String bio, bool needAgree) {
    rpc.send('group-chat-create', [myName, addr, name, bio, needAgree]);
  }

  reSend(int id) {
    //
  }

  messageCreate(MessageType mtype, String content) {
    final gid = this.activedGroup.gid;
    rpc.send('group-chat-message-create', [gid, mtype.toInt(), content]);
  }

  _list(List params) {
    this.clear();
    params.forEach((params) {
        final gc = GroupChat.fromList(params);
        if (gc.isOk) {
          this.orderKeys.add(gc.id);
        } else {
          this.createKeys.add(gc.id);
        }
        this.groups[gc.id] = gc;
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

  _detail(List params) {
    this.clearActivedGroup();
    params[0].forEach((param) {
        this.activedMembers[param[0]] = Member.fromList(param);
    });
    params[1].forEach((param) {
        this.activedMessages[param[0]] = Message.fromList(param);
    });
    notifyListeners();
  }

  _online(List params) {
    final id = params[0];
    if (this.groups.containsKey(id)) {
      this.groups[id].online = true;
      notifyListeners();
    }
  }

  _offline(List params) {
    final id = params[0];
    if (this.groups.containsKey(id)) {
      if (this.groups[id].gid == params[1]) {
        this.groups[id].online = false;
        notifyListeners();
      }
    }
  }

  _memberJoin(List params) {
    //
  }

  _memberOnline(List params) {
    //
  }

  _memberOffline(List params) {
    //
  }

  _messageCreate(List params) {
    final msg = Message.fromList(params);
    if (msg.fid == this.actived) {
      if (!msg.isDelivery) {
        msg.isDelivery = null; // When message create, set is is none;
      }
      this.groups[msg.fid].updateLastMessage(msg, true);
      this.activedMessages[msg.id] = msg;
      rpc.send('group-chat-readed', [this.actived]);
    } else {
      if (this.groups.containsKey(msg.fid)) {
        this.groups[msg.fid].updateLastMessage(msg, false);
      }
    }
    //orderGroups(msg.fid);
    notifyListeners();
  }
}
