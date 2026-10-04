<script setup lang="ts">
import type { FormField } from "~/types/forms";
defineProps<{ fields: FormField[]; edit?: boolean }>();
const model = defineModel<Record<string, unknown>>({ required: true });
const { en } = useLocale();
</script>
<template>
  <template v-for="(field, index) in fields" :key="`${field.key}:${index}`"
    ><label v-if="(!edit || !field.createOnly) && (edit || !field.editOnly)"
      ><template v-if="field.kind === 'checkbox'"
        ><input v-model="model[field.key]" type="checkbox" />{{
          field.label[en ? 1 : 0]
        }}</template
      ><template v-else
        >{{ field.label[en ? 1 : 0]
        }}<textarea
          v-if="field.kind === 'textarea'"
          :value="String(model[field.key] ?? '')"
          :required="field.required"
          @input="
            model[field.key] = ($event.target as HTMLTextAreaElement).value
          " /><select
          v-else-if="field.kind === 'select'"
          v-model="model[field.key]"
          :required="field.required"
        >
          <option
            v-for="option in field.options"
            :key="option.value"
            :value="option.value"
          >
            {{ option.label[en ? 1 : 0] }}
          </option></select
        ><input
          v-else-if="field.kind === 'number'"
          v-model.number="model[field.key]"
          type="number"
          :required="field.required"
          :min="field.min"
          :max="field.max" /><input
          v-else
          v-model="model[field.key]"
          :type="field.kind || 'text'"
          :required="field.required"
          :autocomplete="
            field.kind === 'password' ? 'new-password' : 'off'
          " /></template></label
  ></template>
</template>
