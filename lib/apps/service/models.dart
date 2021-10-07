import 'package:flutter/material.dart';
import 'package:esse/l10n/localizations.dart';

import 'package:esse/apps/assistant/page.dart';
import 'package:esse/apps/file/list.dart';
import 'package:esse/apps/group_chat/list.dart';
import 'package:esse/apps/domain/page.dart';
import 'package:esse/apps/chat/list.dart';

const List<InnerService> INNER_SERVICES = [
  InnerService.Chat,
  InnerService.GroupChat,
  InnerService.Assistant,
  InnerService.Domain,
  //InnerService.Cloud,
];

enum InnerService {
  Chat,
  GroupChat,
  Assistant,
  Domain,
  Cloud,
}

extension InnerServiceExtension on InnerService {
  List<String> params(AppLocalizations lang) {
    switch (this) {
      case InnerService.Chat:
        return [lang.contact, lang.contactIntro, 'assets/logo/logo_chat.png'];
      case InnerService.Assistant:
        return [lang.assistant, lang.assistantBio, 'assets/logo/logo_assistant.png'];
      case InnerService.GroupChat:
        return [lang.groupChat, lang.groupChatIntro, 'assets/logo/logo_group_chat.png'];
      case InnerService.Domain:
        return [lang.domain, lang.domainIntro, 'assets/logo/logo_domain.png'];
      case InnerService.Cloud:
        return [lang.cloud, lang.cloudIntro, 'assets/logo/logo_domain.png'];
    }
  }

  Widget callback() {
    switch (this) {
      case InnerService.Chat:
        return ChatList();
      case InnerService.Assistant:
        return AssistantDetail();
      case InnerService.GroupChat:
        return GroupChatList();
      case InnerService.Domain:
        return DomainDetail();
      case InnerService.Cloud:
        return DomainDetail();
    }
  }
}
