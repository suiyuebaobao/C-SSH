<script setup lang="ts">
import { announcements } from "~/config/resources";
const { t } = useLocale();
</script>
<template>
  <WorkspaceFrame admin
    ><AdminResourceManager :config="announcements" deletable
      ><template #actions="{ row, reload }"
        ><WorkspaceConfirmAction
          v-if="row.status === 'draft'"
          :label="t('发布', 'Publish')"
          :message="
            t(
              '确认向所有客户端发布此公告？',
              'Publish this announcement to all clients?',
            )
          "
          :endpoint="`/api/v1/admin/announcements/${row.id}/publish`"
          method="POST"
          :body="{ expected_revision: row.revision }"
          @done="reload" /></template></AdminResourceManager
    ><AdminNotificationSender
  /></WorkspaceFrame>
</template>
