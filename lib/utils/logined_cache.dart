import 'package:shared_preferences/shared_preferences.dart';

import 'package:esse/account.dart';

const LOGINED_CACHE_NAME = 'logined_account';

/// get all auto-logined account. first one is main.
Future<Account?> getLogined() async {
  SharedPreferences prefs = await SharedPreferences.getInstance();
  final fields = prefs.getStringList(LOGINED_CACHE_NAME);
  if (fields != null && fields.length == 4) {
    return Account(
      fields[0], // pid
      fields[1], // name
      fields[2], // avatar
      fields[3], // pin
    );
  } else {
    prefs.remove(LOGINED_CACHE_NAME);
  }
}

initLogined(Account account) async {
  SharedPreferences prefs = await SharedPreferences.getInstance();
  final List<String> fields = [
    account.pid,
    account.name,
    account.encodeAvatar(),
    account.pin,
  ];
  prefs.setStringList(LOGINED_CACHE_NAME, fields);
}

/// when logout clear all
clearLogined() async {
  SharedPreferences prefs = await SharedPreferences.getInstance();
  prefs.remove(LOGINED_CACHE_NAME);
}
