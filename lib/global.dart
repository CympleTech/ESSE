class Global {
  static String gid = "0000000000000000000000000000000000000000000000000000000000000000";
  static String httpRpc = '127.0.0.1:8000';
  static String wsRpc = '127.0.0.1:8080';
  //static String httpRpc = '192.168.2.148:8001';  // test code
  //static String wsRpc = '192.168.2.148:8081';    // test code
  //static String httpRpc = '192.168.50.250:8001'; // test code
  //static String wsRpc = '192.168.50.250:8081';   // test code
  static String optionCache = 'option';
  static String addr = '0x';

  static String home = '.tdn';
  static String filePath   = home + '/' + gid + '/files/';
  static String imagePath  = home + '/' + gid + '/images/';
  static String thumbPath  = home + '/' + gid + '/thumbs/';
  static String emojiPath  = home + '/' + gid + '/emojis/';
  static String recordPath = home + '/' + gid + '/records/';
  static String avatarPath = home + '/' + gid + '/avatars/';

  static changeGid(String gid) {
    Global.gid = gid;
    Global.filePath   = home + '/' + gid + '/files/';
    Global.imagePath  = home + '/' + gid + '/images/';
    Global.thumbPath  = home + '/' + gid + '/thumbs/';
    Global.emojiPath  = home + '/' + gid + '/emojis/';
    Global.recordPath = home + '/' + gid + '/records/';
    Global.avatarPath = home + '/' + gid + '/avatars/';
  }

  static changeWs(String newWs) {
    Global.wsRpc = newWs;
  }

  static changeHttp(String newHttp) {
    Global.httpRpc = newHttp;
  }

  static printAddr() {
    final len = Global.addr.length;
    if (len > 8) {
      return Global.addr.substring(0, 8) + '...' +  Global.addr.substring(len - 6, len);
    } else {
      return Global.addr;
    }
  }
}
