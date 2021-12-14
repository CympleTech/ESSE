String gidText(String? gid, [String pre='EH']) {
  if (gid == null) {
    return '';
  }
  return pre + gid.toUpperCase();
}

String gidPrint(String? gid, [String pre='EH', int n = 4]) {
  if (gid == null) {
    return '';
  }

  final info = gid.toUpperCase();
  final len = info.length;
  if (len > n+n) {
    return pre + info.substring(0, n) + '...' + info.substring(len - n, len);
  } else {
    return info;
  }
}

String gidParse(String gid, [String pre='EH']) {
  if (gid.length > 2 && gid.substring(0, 2) == pre) {
    return gid.substring(2);
  } else {
    return gid;
  }
}

String addrText(String? addr) {
  if (addr == null) {
    return '';
  }
  return '0x' + addr.toLowerCase();
}

String addrPrint(String? addr) {
  if (addr == null) {
    return '';
  }

  final info = addr.toLowerCase();
  final len = info.length;
  if (len > 12) {
    return '0x' + info.substring(0, 6) + '...' + info.substring(len - 6, len);
  } else {
    return info;
  }
}

String addrParse(String addr) {
  if (addr.length > 2 && addr.substring(0, 2) == '0x') {
    return addr.substring(2);
  } else {
    return addr;
  }
}
