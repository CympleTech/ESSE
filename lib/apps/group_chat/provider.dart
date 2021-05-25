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
  SplayTreeMap<int, Request> requests = SplayTreeMap();

  int actived;
  bool activedOnline;
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
    rpc.addListener('group-chat-join', _join, true);
    rpc.addListener('group-chat-agree', _agree, true);
    rpc.addListener('group-chat-reject', _reject, false);
    rpc.addListener('group-chat-request-list', _requestList, false);
    rpc.addListener('group-chat-member-join', _memberJoin, false);
    rpc.addListener('group-chat-member-info', _memberInfo, false);
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

  create(int gtype, String myName, String addr, String name, String bio, bool needAgree) {
    rpc.send('group-chat-create', [gtype, myName, addr, name, bio, needAgree]);
  }

  reSend(int id, String myName) {
    rpc.send('group-chat-resend', [id, myName]);
  }

  join(String gid, String gaddr, String name, String remark, [String key = '']) {
    rpc.send('group-chat-join', [gid, gaddr, name, remark, key]);
  }

  messageCreate(MessageType mtype, String content) {
    final gid = this.activedGroup.gid;
    rpc.send('group-chat-message-create', [gid, mtype.toInt(), content]);
  }

  requestList(bool all) {
    rpc.send('group-chat-request-list', [all]);
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

  _online(List params) {
    if (this.actived == params[0]) {
      this.online = true;
      notifyListeners();
    }
  }

  _offline(List params) {
    if (this.actived == params[0]) {
      this.online = false;
      notifyListeners();
    }
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

  _join(List params) {
    this.requests[params[0]] = Request.fromList(params);
    notifyListeners();
  }

  _agree(List params) {
    final id = params[0];
    if (this.requests.containsKey(id)) {
      this.requests[id].overIt(false);
    }

    final gc = GroupChat.fromList(params[1]);
    this.orderKeys.add(gc.id);
    this.groups[gc.id] = gc;
    notifyListeners();
  }

  _reject(List params) {
    final id = params[0];
    if (this.requests.containsKey(id)) {
      this.requests[id].overIt(false);
      notifyListeners();
    }
  }

  _requestList(List params) {
    this.requests.clear();
    params.forEach((param) {
        this.requests[param[0]] = Request.fromList(param);
    });
    notifyListeners();
  }

  _memberJoin(List params) {
    final member = Member.fromList(params);
    if (this.actived == member.fid) {
      this.activedMembers[member.id] = member;
      // TODO Better add UI member joined.
      notifyListeners();
    }
  }

  _memberInfo(List params) {
    final id = params[0];
    if (this.activedMembers.containsKey(id)) {
      this.activedMembers[id].addr = params[1];
      this.activedMembers[id].name = params[2];
      notifyListeners();
    }
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
      this.activedMessages[msg.id] = msg;
      notifyListeners();
    }
  }
}
