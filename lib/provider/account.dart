import 'dart:async';
import "dart:collection";
import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'package:esse/models/account.dart';
import 'package:esse/models/friend.dart';
import 'package:esse/models/message.dart';
import 'package:esse/utils/logined_cache.dart';
import 'package:esse/widgets/default_core_show.dart';
import 'package:esse/pages/friend.dart';
import 'package:esse/global.dart';
import 'package:esse/rpc.dart';

const DEFAULT_ONLINE_INIT = 8;
const DEFAULT_ONLINE_DELAY = 5;

class AccountProvider extends ChangeNotifier {
  Map<String, Account> accounts = {}; // account's gid and account.
  String activedAccountId; // actived account gid.
  Account get activedAccount => this.accounts[activedAccountId];

  Map<int, Friend> friends = {}; // all friends. friends need Re-order.
  int activedFriendId; // actived friend's id.
  Friend get activedFriend => this.friends[this.activedFriendId];
  Set<int> topKeys = Set();
  List<int> orderChats = []; // ordered chat friends with last message.

  Map<int, Friend> groups = {}; // all apps.

  /// all requests. request have order.
  SplayTreeMap<int, Request> requests = SplayTreeMap();

  /// current show messages. init number is 100, message have order.
  SplayTreeMap<int, Message> activedMessages = SplayTreeMap();

  /// current user's did.
  String get id => this.activedAccount.id;

  // List<int> get chatKeys => this.orderChats.values.toList();
  List<int> get friendKeys => this.friends.keys.toList(); // TODO

  List<int> get groupKeys => this.groups.keys.toList();

  Widget coreShowWidget = DefaultCoreShow();
  bool systemAppFriendAddNew = false;
  bool systemAppGroupAddNew = false;

  void orderFriends(int id) {
    if (this.orderChats.length == 0 || this.orderChats[0] != id) {
      this.orderChats.remove(id);
      this.orderChats.insert(0, id);
    }
  }

  void addContacts() {
    // TODO order contacts.
  }

  initStates() {
    // rpc
    rpc.addListener('system-info', _systemInfo);
    rpc.addListener('account-update', _accountUpdate);
    rpc.addListener('friend-list', _friendList);
    rpc.addListener('friend-online', _friendOnline);
    rpc.addListener('friend-offline', _friendOffline);
    rpc.addListener('friend-info', _friendInfo);
    rpc.addListener('friend-update', _friendUpdate);
    rpc.addListener('friend-close', _friendClose);
    rpc.addListener('request-list', _requestList);
    rpc.addListener('request-create', _requestCreate);
    rpc.addListener('request-delivery', _requestDelivery);
    rpc.addListener('request-agree', _requestAgree);
    rpc.addListener('request-reject', _requestReject);
    rpc.addListener('request-delete', _requestDelete);
    rpc.addListener('message-list', _messageList);
    rpc.addListener('message-create', _messageCreate);
    rpc.addListener('message-delete', _messageDelete);
    rpc.addListener('message-delivery', _messageDelivery);

    systemInfo();
  }

  /// when security load accounts from rpc.
  initAccounts(Map accounts) {
    this.accounts = accounts;
    initStates();
    initLogined(this.accounts.values.toList());
  }

  /// when security load accounts from cache.
  autoAccounts(String gid, Map accounts) {
    Global.changeGid(gid);
    this.activedAccountId = gid;
    this.accounts = accounts;
    initStates();

    rpc.send('friend-list', []);

    this.activedAccount.online = true;
    rpc.send('account-login', [gid, this.activedAccount.lock]);
    new Future.delayed(Duration(seconds: DEFAULT_ONLINE_INIT),
      () => rpc.send('account-online', [gid]));

    this.accounts.forEach((k, v) {
      if (k != gid && v.online) {
        rpc.send('account-login', [v.gid, v.lock]);
        new Future.delayed(Duration(seconds: DEFAULT_ONLINE_INIT),
            () => rpc.send('account-online', [v.gid]));
      }
    });
  }

  /// when security add account.
  addAccount(Account account) {
    Global.changeGid(account.gid);
    this.activedAccountId = account.gid;
    this.accounts[account.gid] = account;

    rpc.send('account-login', [account.gid, account.lock]);
    new Future.delayed(Duration(seconds: DEFAULT_ONLINE_DELAY),
        () => rpc.send('account-online', [account.gid]));
    updateLogined(account);
  }

  updateActivedAccount(String gid) {
    Global.changeGid(gid);
    this.clearActivedAccount();
    this.activedAccountId = gid;
    this.activedAccount.hasNew = false;

    rpc.send('friend-list', []);

    if (!this.activedAccount.online) {
      this.activedAccount.online = true;
      rpc.send('account-login', [gid, this.activedAccount.lock]);
      new Future.delayed(Duration(seconds: DEFAULT_ONLINE_DELAY),
          () => rpc.send('account-online', [gid]));
    }

    mainLogined(gid);
    notifyListeners();
  }

  logout() {
    this.accounts.clear();
    this.clearActivedAccount();
    rpc.send('account-logout', []);
    clearLogined();
  }

  onlineAccount(String gid, String lock) {
    this.accounts[gid].online = true;
    updateLogined(this.accounts[gid]);

    rpc.send('account-login', [gid, lock]);

    new Future.delayed(Duration(seconds: DEFAULT_ONLINE_DELAY),
        () => rpc.send('account-online', [gid]));
    notifyListeners();
  }

  offlineAccount(String gid) {
    this.accounts[gid].online = false;
    updateLogined(this.accounts[gid]);

    if (gid == this.activedAccountId) {
      this.clearActivedAccount();
    }
    rpc.send('account-offline', [gid]);

    notifyListeners();
  }

  updateActivedApp(Widget widget) {
    this.coreShowWidget = widget;
    this.activedFriendId = null;
    this.activedMessages.clear();
    this.systemAppFriendAddNew = false;
    notifyListeners();
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

  clearActivedAccount() {
    this.topKeys.clear();
    this.orderChats.clear();
    this.friends.clear();
    this.requests.clear();
    this.clearActivedFriend();
  }

  clearActivedFriend() {
    this.activedFriendId = null;
    this.activedMessages.clear();
    this.coreShowWidget = DefaultCoreShow();
  }

  accountUpdate(String name, [Uint8List avatar]) {
    this.activedAccount.name = name;

    if (avatar != null && avatar.length > 0) {
      this.activedAccount.avatar = avatar;
      rpc.send('account-update', [name, this.activedAccount.encodeAvatar()]);
    } else {
      rpc.send('account-update', [name, '']);
    }
    updateLogined(this.activedAccount);

    notifyListeners();
  }

  accountPin(String lock) {
    this.activedAccount.lock = lock;
    updateLogined(this.activedAccount);
    notifyListeners();
  }

  systemInfo() {
    rpc.send('system-info', []);
  }

  /// delete a friend.
  friendUpdate(int id, {String remark, bool isTop}) {
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

    final friend = this.friends[id];
    rpc.send('friend-update', [id, friend.remark, friend.isTop]);
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
    final fgid = this.friends[msg.fid].gid;
    rpc.send('message-create', [msg.fid, fgid, msg.type.toInt(), msg.content]);
    notifyListeners();
  }

  /// delete a message.
  messageDelete(int id) {
    rpc.send('message-delete', [id]);
    this.activedMessages.remove(id);
    notifyListeners();
  }

  // -- callback when receive rpc info. -- //
  _systemInfo(String _gid, List params) {
    Global.addr = '0x' + params[0];
  }

  _accountUpdate(String gid, List params) {
    this.accounts[gid].name = params[0];
    if (params[1].length > 0) {
      if (params[1].length > 1) {
        this.accounts[gid].updateAvatar(params[1]);
      }
    }
    notifyListeners();
  }

  /// list all friends.
  _friendList(String gid, List params) async {
    if (gid == this.activedAccountId) {
      this.orderChats.clear();
      this.friends.clear();

      params.forEach((params) {
          final id = params[0];
          this.friends[id] = Friend.fromList(params);
          this.orderChats.add(id);
          if (this.friends[id].isTop) {
            this.topKeys.add(id);
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
      final id = params[0];
      this.friends[id] = Friend.fromList(params);
      if (this.friends[id].isTop) {
        this.topKeys.add(id);
      }
      notifyListeners();
    }
  }

  _friendUpdate(String gid, List params) async {
    if (gid == this.activedAccountId) {
      final id = params[0];
      if (this.friends.containsKey(id)) {
        this.friends[id].isTop = params[1];
        this.friends[id].remark = params[2];
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
      var friend = Friend.fromList(params[1]);
      this.friends[friend.id] = friend;
      orderFriends(friend.id);
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

  _requestDelete(String gid, List params) async {
    if (gid == this.activedAccountId) {
      this.requests.remove(params[0]);
      notifyListeners();
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
          if (!msg.isDelivery) {
            msg.isDelivery = null; // When message create, set is is none;
          }
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

  _messageDelete(String gid, List params) {
    if (gid == this.activedAccountId) {
      this.activedMessages.remove(params[0]);
      notifyListeners();
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
