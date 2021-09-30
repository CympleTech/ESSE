String betterPrint(String? info) {
  if (info == null) {
    return '';
  }
  final len = info.length;
  if (len > 8) {
    return info.substring(0, 8) + '...' + info.substring(len - 6, len);
  } else {
    return info;
  }
}

String gidPrint(String? gid) {
  if (gid == null) {
    return '';
  }

  final info = gid.toUpperCase();
  final len = info.length;
  if (len > 8) {
    return 'EH' + info.substring(0, 6) + '...' + info.substring(len - 4, len);
  } else {
    return info;
  }
}

String addrPrint(String? addr) {
  if (addr == null) {
    return '';
  }

  final info = addr.toLowerCase();
  final len = info.length;
  if (len > 8) {
    return '0x' + info.substring(0, 8) + '...' + info.substring(len - 6, len);
  } else {
    return info;
  }
}
