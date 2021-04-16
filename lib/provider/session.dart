import 'dart:async';
import "dart:collection";
import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'models/account.dart';
import 'models/friend.dart';
import 'models/message.dart';
import 'models/system.dart';
import 'utils/logined_cache.dart';
import 'widgets/default_core_show.dart';

import 'pages/friend.dart';

import 'rpc.dart';
import 'options.dart';

class SessionProvider extends ChangeNotifier {
  Map<int, Friend> friends = {}; // all friends. friends need Re-order.
  int activedFriendId; // actived friend's id.
  Friend get activedFriend => this.friends[this.activedFriendId];
  Set<int> topKeys = Set();
  List<int> orderChats = []; // ordered chat friends with last message.

  /// all requests. request have order.
  SplayTreeMap<int, Request> requests = SplayTreeMap();

  /// current show messages. init number is 100, message have order.
  SplayTreeMap<int, Message> activedMessages = SplayTreeMap();

  /// current user's did.
  String get id => this.activedAccount.id;

  // List<int> get chatKeys => this.orderChats.values.toList();
  List<int> get friendKeys => this.friends.keys.toList(); // TODO

  void orderFriends(int id) {
    if (this.orderChats.length == 0 || this.orderChats[0] != id) {
      this.orderChats.remove(id);
      this.orderChats.insert(0, id);
    }
  }

  void addContacts() {
    // TODO order contacts.
  }

  init() {
    // rpc
    rpc.addListener('friend-list', _friendList);
    rpc.addListener('friend-online', _friendOnline);
    rpc.addListener('friend-offline', _friendOffline);
    rpc.addListener('friend-info', _friendInfo);
    rpc.addListener('friend-close', _friendClose);
    rpc.addListener('request-list', _requestList);
    rpc.addListener('request-create', _requestCreate);
    rpc.addListener('request-delivery', _requestDelivery);
    rpc.addListener('request-agree', _requestAgree);
    rpc.addListener('request-reject', _requestReject);
    rpc.addListener('message-list', _messageList);
    rpc.addListener('message-create', _messageCreate);
    rpc.addListener('message-delivery', _messageDelivery);
  }

  updateActivedFriend(int id, bool isDesktop) {
    this.activedFriendId = id;
    this.activedMessages.clear();
    this.friends[id].lastMessageReaded = true;
    rpc.send('message-list', [this.activedFriendId]);

    if (isDesktop && this.coreShowWidget is! ChatPage) {
      this.coreShowWidget = ChatPage();
    }
    notifyListeners();
  }

  clearActivedFriend() {
    this.activedFriendId = null;
    this.activedMessages.clear();
    this.coreShowWidget = DefaultCoreShow();
  }

  /// delete a friend.
  friendInfo(int id, {String remark, bool isTop, bool isHidden}) {
    if (remark != null) {
      this.friends[id].remark = remark;
    }

    if (isTop != null) {
      this.friends[id].isTop = isTop;
      if (isTop) {
        this.topKeys.add(id);
      } else {
        this.topKeys.remove(id);
      }
    }

    if (isHidden != null) {
      this.friends[id].isHidden = isHidden;
    }

    final friend = this.friends[id];
    rpc.send('friend-info', [id, friend.remark, friend.isTop, friend.isHidden]);
    notifyListeners();
  }

  /// delete a friend.
  friendClose(int id) {
    this.friends[id].isClosed = true;
    this.friends[id].online = false;
    rpc.send('friend-close', [id]);
    notifyListeners();
  }

  /// delete a friend.
  friendDelete(int id) {
    if (id == this.activedFriendId) {
      this.activedFriendId = null;
      this.activedMessages.clear();
      this.coreShowWidget = DefaultCoreShow();
    }
    this.friends.remove(id);
    this.orderChats.remove(id);
    this.topKeys.remove(id);
    rpc.send('friend-delete', [id]);
    notifyListeners();
  }

  /// list all request.
  requestList() {
    rpc.send('request-list', []);
  }

  /// clear the memory requests.
  requestClear() {
    this.requests.clear();
  }

  /// create a request for friend.
  requestCreate(Request req) {
    rpc.send('request-create', [req.gid, req.addr, req.name, req.remark]);
    this.requests.remove(req.id);
    rpc.send('request-list', []);
    notifyListeners();
  }

  /// agree a request for friend.
  requestAgree(int id) {
    rpc.send('request-agree', [id]);
    this.requests[id].overIt(true);
    notifyListeners();
  }

  /// reject a request for friend.
  requestReject(int id) {
    rpc.send('request-reject', [id]);
    this.requests[id].overIt(false);
    notifyListeners();
  }

  /// delte a request for friend.
  requestDelete(int id) {
    rpc.send('request-delete', [id]);
    this.requests.remove(id);
    notifyListeners();
  }

  /// create a message. need core server handle this message.
  /// and then this message will show in message list.
  messageCreate(Message msg) {
    rpc.send('message-create', [msg.fid, msg.type.toInt(), msg.content]);
    notifyListeners();
  }

  /// delete a message.
  messageDelete(int id) {
    rpc.send('message-delete', [id]);
    this.activedMessages.remove(id);
    notifyListeners();
  }

  // -- callback when receive rpc info. -- //

  /// list all friends.
  _friendList(String gid, List params) async {
    if (gid == this.activedAccountId) {
      this.orderChats.clear();
      this.friends.clear();

      params.forEach((params) {
        if (params.length == 12) {
          final id = params[0];
          this.friends[id] = Friend.fromList(params);
          this.orderChats.add(id);
          if (this.friends[id].isTop) {
            this.topKeys.add(id);
          }
        }
      });
      notifyListeners();
    }
  }

  _friendOnline(String gid, List params) async {
    if (gid == this.activedAccountId) {
      final id = params[0];
      if (this.friends.containsKey(id)) {
        this.friends[id].online = true;
        this.friends[id].addr = params[1];
        notifyListeners();
      }
    }
  }

  _friendOffline(String gid, List params) async {
    if (gid == this.activedAccountId) {
      final id = params[0];
      if (this.friends.containsKey(id)) {
        this.friends[id].online = false;
        notifyListeners();
      }
    }
  }

  _friendInfo(String gid, List params) async {
    if (gid == this.activedAccountId) {
      if (params.length == 12) {
        final id = params[0];
        this.friends[id] = Friend.fromList(params);
        if (this.friends[id].isTop) {
          this.topKeys.add(id);
        }
        notifyListeners();
      }
    }
  }

  _friendClose(String gid, List params) async {
    if (gid == this.activedAccountId) {
      final id = params[0];
      if (this.friends.containsKey(id)) {
        this.friends[id].isClosed = true;
        this.friends[id].online = false;
        notifyListeners();
      }
    }
  }

  /// list requests for friend.
  _requestList(String gid, List params) async {
    if (gid == this.activedAccountId) {
      this.requests.clear();
      params.forEach((param) {
        if (param.length == 10) {
          this.requests[param[0]] = Request.fromList(param);
        }
      });
      notifyListeners();
    }
  }

  /// receive a request for friend.
  _requestCreate(String gid, List params) async {
    if (gid == this.activedAccountId) {
      if (params.length == 10) {
        this.requests[params[0]] = Request.fromList(params);
        this.systemAppFriendAddNew = true;
        notifyListeners();
      }
    } else {
      if (this.accounts.containsKey(gid)) {
        this.accounts[gid].hasNew = true;
        notifyListeners();
      }
    }
  }

  /// created request had delivery.
  _requestDelivery(String gid, List params) async {
    if (gid == this.activedAccountId) {
      final id = params[0];
      final isDelivery = params[1];
      if (this.requests.containsKey(id)) {
        this.requests[id].isDelivery = isDelivery;
        notifyListeners();
      }
    }
  }

  /// request for friend receive agree.
  _requestAgree(String gid, List params) async {
    if (gid == this.activedAccountId) {
      final id = params[0]; // request's id.
      if (this.requests.containsKey(id)) {
        this.requests[id].overIt(true);
      }
      if (params[1].length == 12) {
        var friend = Friend.fromList(params[1]);
        this.friends[friend.id] = friend;
        orderFriends(friend.id);
      }
      notifyListeners();
    } else {
      if (this.accounts.containsKey(gid)) {
        this.accounts[gid].hasNew = true;
        notifyListeners();
      }
    }
  }

  /// request for friend receive reject.
  _requestReject(String gid, List params) async {
    if (gid == this.activedAccountId) {
      final id = params[0];
      if (this.requests.containsKey(id)) {
        this.requests[id].overIt(false);
        notifyListeners();
      }
    }
  }

  /// list message with friend.
  _messageList(String gid, List params) async {
    if (gid == this.activedAccountId) {
      params.forEach((param) {
        if (param.length == 8) {
          this.activedMessages[param[0]] = Message.fromList(param);
        }
      });
      notifyListeners();
    }
  }

  /// friend send message to me.
  _messageCreate(String gid, List params) async {
    if (gid == this.activedAccountId) {
      if (params.length == 8) {
        final msg = Message.fromList(params);
        if (msg.fid == this.activedFriendId) {
          this.friends[msg.fid].updateLastMessage(msg, true);
          this.activedMessages[msg.id] = msg;
          rpc.send('friend-readed', [this.activedFriendId]);
        } else {
          if (this.friends.containsKey(msg.fid)) {
            this.friends[msg.fid].updateLastMessage(msg, false);
          }
        }
        orderFriends(msg.fid);
        notifyListeners();
      }
    } else {
      if (this.accounts.containsKey(gid)) {
        this.accounts[gid].hasNew = true;
        notifyListeners();
      }
    }
  }

  /// created message had delivery.
  _messageDelivery(String gid, List params) async {
    if (gid == this.activedAccountId) {
      final id = params[0];
      final isDelivery = params[1];
      if (this.activedMessages.containsKey(id)) {
        this.activedMessages[id].isDelivery = isDelivery;
        notifyListeners();
      }
    }
  }
}
