<script setup lang="ts">
import { contentLabels } from "~/config/content-labels";
import type { ContentNode as Node } from "~/types/forms";
const props = defineProps<{ value: Node; name: string }>();
const emit = defineEmits<{ change: [value: Node] }>();
const { en } = useLocale();
const label = computed(
  () => contentLabels[props.name]?.[en.value ? 1 : 0] ?? props.name,
);
const editable = (key: string) => key in contentLabels;
function changed(key: string | number, value: Node) {
  const copy = Array.isArray(props.value)
    ? [...props.value]
    : { ...(props.value as Record<string, Node>) };
  (copy as Record<string | number, Node>)[key] = value;
  emit("change", copy);
}
</script>
<template>
  <fieldset v-if="Array.isArray(value)" class="content-fields">
    <legend>{{ label }}</legend>
    <details v-for="(item, index) in value" :key="index">
      <summary>{{ label }} {{ index + 1 }}</summary>
      <AdminContentFields
        :value="item"
        :name="name"
        @change="changed(index, $event)"
      />
    </details>
  </fieldset>
  <fieldset
    v-else-if="value && typeof value === 'object'"
    class="content-fields"
  >
    <legend>{{ label }}</legend>
    <template v-for="(item, key) in value" :key="key"
      ><AdminContentFields
        v-if="editable(String(key))"
        :value="item"
        :name="String(key)"
        @change="changed(String(key), $event)"
    /></template>
  </fieldset>
  <label v-else-if="typeof value === 'string'"
    >{{ label
    }}<textarea
      v-if="
        [
          'body',
          'lead',
          'answer',
          'meta_description',
          'footer_summary',
        ].includes(name) || value.length > 100
      "
      :value="value"
      @input="
        emit('change', ($event.target as HTMLTextAreaElement).value)
      " /><input
      v-else
      :value="value"
      @input="emit('change', ($event.target as HTMLInputElement).value)"
  /></label>
</template>
