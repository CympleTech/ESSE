import 'package:flutter/material.dart';
import 'package:esse/l10n/localizations.dart';

import 'package:esse/apps/assistant/page.dart';
import 'package:esse/apps/file/page.dart';
import 'package:esse/apps/group_chat/page.dart';

enum InnerService {
  Files,
  Assistant,
  GroupChat,
}

extension InnerServiceExtension on InnerService {
  List<String> params(AppLocalizations lang) {
    switch (this) {
      case InnerService.Files:
        return [lang.files, lang.filesBio, 'assets/logo/logo_files.png'];
      case InnerService.Assistant:
        return [lang.assistant, lang.assistantBio, 'assets/logo/logo_assistant.png'];
      case InnerService.GroupChat:
        return [lang.groupChat, lang.groupChatIntro, 'assets/logo/logo_group_chat.png'];
    }
  }

  Widget callback() {
    switch (this) {
      case InnerService.Files:
        return FolderList();
      case InnerService.Assistant:
        return AssistantDetail();
      case InnerService.GroupChat:
        return GroupChatList();
    }
  }
}
