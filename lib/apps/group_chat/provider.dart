import "dart:collection";

import 'package:flutter/material.dart';

import 'package:esse/rpc.dart';
import 'package:esse/global.dart';

import 'package:esse/apps/primitives.dart';
import 'package:esse/apps/group_chat/models.dart';

class GroupChatProvider extends ChangeNotifier {
  List<GroupType> createSupported = [GroupType.Encrypted, GroupType.Private, GroupType.Open];
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

  List activedMemberOrder(String myId) {
    int meId = 0;
    bool meOwner = false;
    bool meManager = false;
    List<int> allKeys = [];
    List<int> managers = [];
    List<int> commons = [];
    List<int> blocks = [];
    this.activedMembers.forEach((i, m) {
        if (m.mid == myId) {
          meId = i;
          if (m.isManager) {
            meManager = true;
            if (m.mid == this.activedGroup.owner) {
              meOwner = true;
            }
          }
        } else {
          if (m.isBlock) {
            blocks.add(i);
          } else {
            if (m.isManager) {
              if (m.mid == this.activedGroup.owner) {
                allKeys.add(i);
              } else {
                managers.add(i);
              }
            } else {
              commons.add(i);
            }
          }
        }
    });
    return [allKeys + managers + commons + blocks, meId, meOwner, meManager];
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
    rpc.addListener('group-chat-request-list', _requestList, false);
    rpc.addListener('group-chat-request-handle', _requestHandle, false);
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

  join(GroupType gtype, String gid, String gaddr, String name, String remark, [String proof = '', String key = '']) {
    rpc.send('group-chat-join', [gtype.toInt(), gid, gaddr, name, remark, proof, key]);
  }

  requestHandle(String gid, int id, int rid, bool ok) {
    rpc.send('group-chat-request-handle', [gid, id, rid, ok]);
  }

  requestBlock(String gid, String rgid) {
    rpc.send('group-chat-request-block', [gid, rgid]);
  }

  messageCreate(MessageType mtype, String content) {
    final gid = this.activedGroup.gid;
    rpc.send('group-chat-message-create', [gid, mtype.toInt(), content]);
  }

  requestList(bool all) {
    rpc.send('group-chat-request-list', [all]);
  }

  close(int id) {
    // rpc.send('group-chat-close', [id]);
  }

  delete(int id) {
    // rpc.send('group-chat-delete', [id]);
  }

  reAdd(int id) {
    // rpc.send('group-chat-readd', [id]);
  }

  invite(String gid, List<int> ids) {
    print(gid);
    print(ids);
    rpc.send('group-chat-invite', [ids]);
  }

  memberUpdate(int id, bool isBlock) {
    rpc.send('group-chat-member-update', [id, isBlock]);
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
      this.activedOnline = true;
      notifyListeners();
    }
  }

  _offline(List params) {
    if (this.actived == params[0]) {
      this.activedOnline = false;
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

  _requestHandle(List params) {
    final id = params[0];
    final ok = params[1];
    if (this.requests.containsKey(id)) {
      this.requests[id].overIt(ok);
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
    final id = params[0];
    final mgid = params[1];
    final maddr = params[2];

    if (this.actived == id) {
      int mid;
      this.activedMembers.forEach((i, m) {
          if (m.mid == mgid) {
            mid = i;
          }
      });
      if (mid != null) {
        this.activedMembers[mid].addr = maddr;
        this.activedMembers[mid].online = true;
        notifyListeners();
      }
    }
  }

  _memberOffline(List params) {
    final id = params[0];
    final mgid = params[1];
    if (this.actived == id) {
      int mid;
      this.activedMembers.forEach((i, m) {
          if (m.mid == mgid) {
            mid = i;
          }
      });
      if (mid != null) {
        this.activedMembers[mid].online = false;
        notifyListeners();
      }
    }
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
