import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/assistant/page.dart';
import 'package:esse/apps/file/page.dart';

enum InnerService {
  Files,
  Assistant,
}

extension InnerServiceExtension on InnerService {
  List<String> params(AppLocalizations lang) {
    switch (this) {
      case InnerService.Files:
        return [lang.files, lang.filesBio, 'assets/logo/logo_files.png'];
      case InnerService.Assistant:
        return [lang.assistant, lang.assistantBio, 'assets/logo/logo_assistant.png'];
    }
  }

  void callback(context, isDesktop, lang) {
    Widget coreWidget = null;
    String listTitle = null;
    Widget listHome = null;

    if (isDesktop) {
      switch (this) {
        case InnerService.Files:
          listTitle = lang.files;
          listHome = FolderList();
          break;
        case InnerService.Assistant:
          coreWidget = AssistantDetail();
          break;
      }
      Provider.of<AccountProvider>(context, listen: false).updateActivedApp(coreWidget, listTitle, listHome);
    } else {
      switch (this) {
        case InnerService.Files:
          Provider.of<AccountProvider>(context, listen: false).updateActivedApp(null, lang.files, FolderList());
          break;
        case InnerService.Assistant:
          Navigator.push(context, MaterialPageRoute(builder: (_) => AssistantPage()));
          break;
      }
    }
  }
}
