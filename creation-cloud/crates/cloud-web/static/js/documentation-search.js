// 文档目录筛选：仅按当前页面的章节标题过滤目录链接，不处理正文或外部搜索。
(() => {
  "use strict";

  const normalize = (value) => value.normalize("NFKC").trim().toLocaleLowerCase();

  document.querySelectorAll("[data-doc-index]").forEach((index) => {
    const input = index.querySelector("[data-doc-search]");
    const links = Array.from(index.querySelectorAll("[data-doc-link]"));
    const groups = Array.from(index.querySelectorAll("[data-doc-group]"));
    const result = index.querySelector("[data-doc-result]");
    const empty = index.querySelector("[data-doc-empty]");

    if (!(input instanceof HTMLInputElement) || links.length === 0) {
      return;
    }

    const applyFilter = () => {
      const query = normalize(input.value);
      let matches = 0;

      links.forEach((link) => {
        const title = normalize(link.dataset.docTitle || "");
        const visible = query === "" || title.includes(query);
        link.hidden = !visible;
        if (visible) {
          matches += 1;
        }
      });

      groups.forEach((group) => {
        group.hidden = !Array.from(group.querySelectorAll("[data-doc-link]")).some(
          (link) => !link.hidden,
        );
      });

      if (result) {
        result.textContent = `${matches} / ${links.length}`;
      }
      if (empty) {
        empty.hidden = matches !== 0;
      }
    };

    input.addEventListener("input", applyFilter);
    input.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && input.value !== "") {
        input.value = "";
        applyFilter();
      }
    });
    applyFilter();
  });
})();
