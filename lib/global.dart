class Global {
  static String version = 'v0.5.0';
  static String pid = '0000000000000000000000000000000000000000000000000000000000000000';
  static String httpRpc = '127.0.0.1:7365';
  static String wsRpc = '127.0.0.1:7366';
  //static String httpRpc = '192.168.2.148:8001';  // test code
  //static String wsRpc = '192.168.2.148:8081';    // test code
  //static String httpRpc = '192.168.50.250:8001'; // test code
  //static String wsRpc = '192.168.50.250:8081';   // test code
  static String optionCache = 'option';

  static String home = '.tdn';
  static String filePath   = home + '/' + pid + '/files/';
  static String imagePath  = home + '/' + pid + '/images/';
  static String thumbPath  = home + '/' + pid + '/thumbs/';
  static String emojiPath  = home + '/' + pid + '/emojis/';
  static String recordPath = home + '/' + pid + '/records/';
  static String avatarPath = home + '/' + pid + '/avatars/';

  static changePid(String pid) {
    Global.pid = pid;
    Global.filePath   = home + '/' + pid + '/files/';
    Global.imagePath  = home + '/' + pid + '/images/';
    Global.thumbPath  = home + '/' + pid + '/thumbs/';
    Global.emojiPath  = home + '/' + pid + '/emojis/';
    Global.recordPath = home + '/' + pid + '/records/';
    Global.avatarPath = home + '/' + pid + '/avatars/';
  }

  static changeWs(String newWs) {
    Global.wsRpc = newWs;
  }

  static changeHttp(String newHttp) {
    Global.httpRpc = newHttp;
  }
}
