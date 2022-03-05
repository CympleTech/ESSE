import 'package:shared_preferences/shared_preferences.dart';

import 'package:esse/account.dart';

const LOGINED_CACHE_NAME = 'logined';

/// get all auto-logined account. first one is main.
Future<List<Account>> getLogined() async {
  List<Account> accounts = [];
  SharedPreferences prefs = await SharedPreferences.getInstance();
  final ids = prefs.getStringList(LOGINED_CACHE_NAME);
  if (ids != null) {
    ids.forEach((id) {
      final fields = prefs.getStringList(id);
      if (fields != null && fields.length == 5) {
        accounts.add(Account(
          fields[0], // pid
          fields[1], // name
          fields[2], // avatar
          false,
        ));
      } else {
        prefs.remove(id);
      }
    });
  }

  return accounts;
}

initLogined(String pid, List<Account> accounts) async {
  SharedPreferences prefs = await SharedPreferences.getInstance();
  final ids = prefs.getStringList(LOGINED_CACHE_NAME);
  if (ids != null) {
    ids.forEach((id) {
      prefs.remove(id);
    });
  }

  List<String> newIds = [pid];
  accounts.forEach((account) {
    final List<String> fields = [
      account.pid,
      account.name,
      account.encodeAvatar(),
    ];

    if (account.pid != pid) {
      newIds.add(account.pid);
    }

    prefs.setStringList(account.pid, fields);
  });

  prefs.setStringList(LOGINED_CACHE_NAME, newIds);
}

/// update auto-logined account.
updateLogined(Account account) async {
  SharedPreferences prefs = await SharedPreferences.getInstance();
  List<String>? ids = prefs.getStringList(LOGINED_CACHE_NAME);
  if (ids == null) {
    ids = [];
  }


  if (!ids.contains(account.pid)) {
    ids.add(account.pid);
    prefs.setStringList(LOGINED_CACHE_NAME, ids);
  }

  final List<String> fields = [
    account.pid,
    account.name,
    account.encodeAvatar(),
  ];

  prefs.setStringList(account.pid, fields);
}

/// change main logined account.
mainLogined(String pid) async {
  SharedPreferences prefs = await SharedPreferences.getInstance();
  List<String>? ids = prefs.getStringList(LOGINED_CACHE_NAME);
  if (ids == null) {
    ids = [];
  }

  ids.remove(pid);
  ids.insert(0, pid);
  prefs.setStringList(LOGINED_CACHE_NAME, ids);
}

/// remove auto-login accounts.
removeLogined(String pid) async {
  SharedPreferences prefs = await SharedPreferences.getInstance();
  prefs.remove(pid);
  List<String>? ids = prefs.getStringList(LOGINED_CACHE_NAME);
  if (ids == null) {
    ids = [];
  }

  if (ids.contains(pid)) {
    ids.remove(pid);
    prefs.setStringList(LOGINED_CACHE_NAME, ids);
  }
}

/// when logout clear all
clearLogined() async {
  SharedPreferences prefs = await SharedPreferences.getInstance();
  List<String>? ids = prefs.getStringList(LOGINED_CACHE_NAME);
  if (ids == null) {
    ids = [];
  }

  ids.forEach((id) {
    prefs.remove(id);
  });
  prefs.remove(LOGINED_CACHE_NAME);
}
