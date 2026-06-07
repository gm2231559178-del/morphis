# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: tests/crud.spec.ts >> Full CRUD flow >> create, view, edit, and delete a material
- Location: tests/crud.spec.ts:4:7

# Error details

```
Error: expect(received).toContain(expected) // indexOf

Expected substring: "E2E_1780826060790"
Received string:    "Morphis AdminColorwaysFeature AttributesMaterial FeaturesMaterialsprotected dataSizesuser permissionsEN中文EntitiesMaterialsNewNew MaterialsCreate a new recordMat. No.*Name*Status*-- Select --ActiveDiscontinued[Network] Internal Server ErrorCreateself.__next_r=\"jxmTCXQkL2CDpQqU5dwZo\"(self.__next_f=self.__next_f||[]).push([0])self.__next_f.push([1,\"9:I[\\\"[project]/node_modules/next/dist/next-devtools/userspace/app/segment-explorer-node.js [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"SegmentViewNode\\\"]\\nb:\\\"$Sreact.fragment\\\"\\n1f:I[\\\"[project]/components/theme-provider.tsx [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"ThemeProvider\\\"]\\n21:I[\\\"[project]/components/locale-provider.tsx [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"LocaleProvider\\\"]\\n23:I[\\\"[project]/node_modules/next-auth/react.js [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"SessionProvider\\\"]\\n25:I[\\\"[project]/lib/client.tsx [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"GraphQLProvider\\\"]\\n27:I[\\\"[project]/components/nav-bar.tsx [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"NavBar\\\"]\\n2a:I[\\\"[project]/node_modules/next/dist/client/components/layout-router.js [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"default\\\"]\\n2c:I[\\\"[project]/node_modules/next/dist/client/components/render-from-template-context.js [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"default\\\"]\\n3e:I[\\\"[project]/components/toast.tsx [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"ToastContainer\\\"]\\n48:I[\\\"[project]/node_modules/next/dist/client/components/client-page.js [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"ClientPageRoot\\\"]\\n49:I[\\\"[project]/app/[entity]/new/page.tsx [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\",\\\"/_next/static/chunks/_0u99gvz._.js\\\",\\\"/_next/static/chunks/app_%5Bentity%5D_new_page_tsx_143_m3u._.js\\\"],\\\"default\\\"]\\n51:I[\\\"[project]/node_modules/next/dist/lib/framework/boundary-components.js [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"OutletBoundary\\\"]\\n53:\\\"$Sreact.suspense\\\"\\n61:I[\\\"[project]/node_modules/next/dist/lib/framework/boundary-components.js [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"ViewportBoundary\\\"]\\n6b:I[\\\"[project]/node_modules/next/dist/lib/framework/boundary-components.js [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"MetadataBoundary\\\"]\\n72:I[\\\"[project]/node_modules/next/dist/client/components/builtin/global-error.js [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\",\\\"/_next/static/chunks/node_modules_next_dist_client_components_builtin_global-error_143_m3u.js\\\"],\\\"default\\\",1]\\n81:I[\\\"[project]/node_modules/next/dist/lib/metadata/generate/icon-mark.js [app-client] (ecmascript)\\\",[\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"/_next/s\"])self.__next_f.push([1,\"tatic/chunks/app_layout_tsx_007e4b2._.js\\\"],\\\"IconMark\\\"]\\n:HL[\\\"/_next/static/chunks/%5Broot-of-the-server%5D__0cbk-n2._.css\\\",\\\"style\\\"]\\n:HL[\\\"/_next/static/media/797e433ab948586e-s.p.0w5z4e7s8jfe5.woff2\\\",\\\"font\\\",{\\\"crossOrigin\\\":\\\"\\\",\\\"type\\\":\\\"font/woff2\\\"}]\\n:HL[\\\"/_next/static/media/caa3a2e1cccd8315-s.p.0wgildi0cnwt9.woff2\\\",\\\"font\\\",{\\\"crossOrigin\\\":\\\"\\\",\\\"type\\\":\\\"font/woff2\\\"}]\\n1:D\\\"$6\\\"\\n1:D\\\"$2\\\"\\n1:D\\\"$7\\\"\\n1:null\\n10:D\\\"$1a\\\"\\n10:D\\\"$11\\\"\\n10:D\\\"$1c\\\"\\n2e:D\\\"$30\\\"\\n2e:D\\\"$2f\\\"\\n2e:D\\\"$32\\\"\\n2e:D\\\"$31\\\"\\n2e:D\\\"$33\\\"\\n2e:[[\\\"$\\\",\\\"title\\\",null,{\\\"children\\\":\\\"404: This page could not be found.\\\"},\\\"$31\\\",\\\"$34\\\",1],[\\\"$\\\",\\\"div\\\",null,{\\\"style\\\":{\\\"fontFamily\\\":\\\"system-ui,\\\\\\\"Segoe UI\\\\\\\",Roboto,Helvetica,Arial,sans-serif,\\\\\\\"Apple Color Emoji\\\\\\\",\\\\\\\"Segoe UI Emoji\\\\\\\"\\\",\\\"height\\\":\\\"100vh\\\",\\\"textAlign\\\":\\\"center\\\",\\\"display\\\":\\\"flex\\\",\\\"flexDirection\\\":\\\"column\\\",\\\"alignItems\\\":\\\"center\\\",\\\"justifyContent\\\":\\\"center\\\"},\\\"children\\\":[\\\"$\\\",\\\"div\\\",null,{\\\"children\\\":[[\\\"$\\\",\\\"style\\\",null,{\\\"dangerouslySetInnerHTML\\\":{\\\"__html\\\":\\\"body{color:#000;background:#fff;margin:0}.next-error-h1{border-right:1px solid rgba(0,0,0,.3)}@media (prefers-color-scheme:dark){body{color:#fff;background:#000}.next-error-h1{border-right:1px solid rgba(255,255,255,.3)}}\\\"}},\\\"$31\\\",\\\"$37\\\",1],[\\\"$\\\",\\\"h1\\\",null,{\\\"className\\\":\\\"next-error-h1\\\",\\\"style\\\":{\\\"display\\\":\\\"inline-block\\\",\\\"margin\\\":\\\"0 20px 0 0\\\",\\\"padding\\\":\\\"0 23px 0 0\\\",\\\"fontSize\\\":24,\\\"fontWeight\\\":500,\\\"verticalAlign\\\":\\\"top\\\",\\\"lineHeight\\\":\\\"49px\\\"},\\\"children\\\":404},\\\"$31\\\",\\\"$38\\\",1],[\\\"$\\\",\\\"div\\\",null,{\\\"style\\\":{\\\"display\\\":\\\"inline-block\\\"},\\\"children\\\":[\\\"$\\\",\\\"h2\\\",null,{\\\"style\\\":{\\\"fontSize\\\":14,\\\"fontWeight\\\":400,\\\"lineHeight\\\":\\\"49px\\\",\\\"margin\\\":0},\\\"children\\\":\\\"This page could not be found.\\\"},\\\"$31\\\",\\\"$3a\\\",1]},\\\"$31\\\",\\\"$39\\\",1]]},\\\"$31\\\",\\\"$36\\\",1]},\\\"$31\\\",\\\"$35\\\",1]]\\n10:[\\\"$\\\",\\\"html\\\",null,{\\\"lang\\\":\\\"en\\\",\\\"className\\\":\\\"geist_a71539c9-module__T19VSG__variable geist_mono_8d43a2aa-module__8Li5zG__variable h-full antialiased\\\",\\\"children\\\":[\\\"$\\\",\\\"body\\\",null,{\\\"className\\\":\\\"min-h-full flex flex-col\\\",\\\"children\\\":[\\\"$\\\",\\\"$L1f\\\",null,{\\\"children\\\":[\\\"$\\\",\\\"$L21\\\",null,{\\\"children\\\":[\\\"$\\\",\\\"$L23\\\",null,{\\\"children\\\":[\\\"$\\\",\\\"$L25\\\",null,{\\\"children\\\":[[\\\"$\\\",\\\"$L27\\\",null,{},\\\"$11\\\",\\\"$26\\\",1],[\\\"$\\\",\\\"main\\\",null,{\\\"className\\\":\\\"flex-1 px-6 py-6 max-w-6xl w-full mx-auto\\\",\\\"children\\\":[\\\"$\\\",\\\"$L2a\\\",null,{\\\"parallelRouterKey\\\":\\\"children\\\",\\\"error\\\":\\\"$undefined\\\",\\\"errorStyles\\\":\\\"$undefined\\\",\\\"errorScripts\\\":\\\"$undefined\\\",\\\"template\\\":[\\\"$\\\",\\\"$L2c\\\",null,{},null,\\\"$2b\\\",1],\\\"templateStyles\\\":\\\"$undefined\\\",\\\"templateScripts\\\":\\\"$undefined\\\",\\\"notFound\\\":[\\\"$\\\",\\\"$L9\\\",\\\"c-not-found\\\",{\\\"type\\\":\\\"not-found\\\",\\\"pagePath\\\":\\\"__next_builtin__not-found.js\\\",\\\"children\\\":[\\\"$2e\\\",[]]},null,\\\"$2d\\\",0],\\\"forbidden\\\":\\\"$undefined\\\",\\\"unauthorized\\\":\\\"$undefined\\\",\\\"segmentViewBoundaries\\\":[[\\\"$\\\",\\\"$L9\\\",null,{\\\"type\\\":\\\"boundary:not-found\\\",\\\"pagePath\\\":\\\"__next_builtin__not-found.js@boundary\\\"},null,\\\"$3b\\\",1],\\\"$undefined\\\",\\\"$undefined\\\",[\\\"$\\\",\\\"$L9\\\",null,{\\\"type\\\":\\\"boundary:global-error\\\",\\\"pagePath\\\":\\\"__next_builtin__global-error.js\\\"},null,\\\"$3c\\\",1]]},null,\\\"$29\\\",1]},\\\"$11\\\",\\\"$28\\\",1],[\\\"$\\\",\\\"$L3e\\\",null,{},\\\"$11\\\",\\\"$3d\\\",1]]},\\\"$11\\\",\\\"$24\\\",1]},\\\"$11\\\",\\\"$22\\\",1]},\\\"$11\\\",\\\"$20\\\",1]},\\\"$11\\\",\\\"$1e\\\",1]},\\\"$11\\\",\\\"$1d\\\",1]},\\\"$11\\\",\\\"$1b\\\",1]\\n4c:D\\\"$4e\\\"\\n4c:D\\\"$4d\\\"\\n4c:D\\\"$50\\\"\\n4c:[\\\"$\\\",\\\"$L51\\\",null,{\\\"children\\\":[\\\"$\\\",\\\"$53\\\",null,{\\\"name\\\":\\\"Next.MetadataOutlet\\\",\\\"children\\\":\\\"$@54\\\"},\\\"$4d\\\",\\\"$52\\\",1]},\\\"$4d\\\",\\\"$4f\\\",1]\\n57:D\\\"$5a\\\"\\n57:D\\\"$58\\\"\\n57:D\\\"$5b\\\"\\n57:null\\n5c:D\\\"$5e\\\"\\n5c:D\\\"$5d\\\"\\n5c:D\\\"$60\\\"\\n62:D\\\"$64\\\"\\n62:D\\\"$63\\\"\\n5c:[\\\"$\\\",\\\"$L61\\\",null,{\\\"children\\\":\\\"$L62\\\"},\\\"$5d\\\",\\\"$5f\\\",1]\\n65:D\\\"$67\\\"\\n65:D\\\"$66\\\"\\n65:D\\\"$69\\\"\\n6d:D\\\"$6f\\\"\\n6d:D\\\"$6e\\\"\\n65:[\\\"$\\\",\\\"div\\\",null,{\\\"hidden\\\":true,\\\"children\\\":[\\\"$\\\",\\\"$L6b\\\",null,{\\\"children\\\":[\\\"$\\\",\\\"$53\\\",null,{\\\"name\\\":\\\"Next.Metadata\\\",\\\"children\\\":\\\"$L6d\\\"},\\\"$66\\\",\\\"$6c\\\",1]},\\\"$66\\\",\\\"$6a\\\",1]},\\\"$66\\\",\\\"$68\\\",1]\\n71:[]\\n\"])self.__next_f.push([1,\"0:{\\\"P\\\":\\\"$1\\\",\\\"c\\\":[\\\"\\\",\\\"materials\\\",\\\"new\\\"],\\\"q\\\":\\\"\\\",\\\"i\\\":true,\\\"f\\\":[[[\\\"\\\",{\\\"children\\\":[[\\\"entity\\\",\\\"materials\\\",\\\"d\\\",null],{\\\"children\\\":[\\\"new\\\",{\\\"children\\\":[\\\"__PAGE__\\\",{}]}]}]},\\\"$undefined\\\",\\\"$undefined\\\",16],[[\\\"$\\\",\\\"$L9\\\",\\\"layout\\\",{\\\"type\\\":\\\"layout\\\",\\\"pagePath\\\":\\\"layout.tsx\\\",\\\"children\\\":[\\\"$\\\",\\\"$b\\\",\\\"c\\\",{\\\"children\\\":[[[\\\"$\\\",\\\"link\\\",\\\"0\\\",{\\\"rel\\\":\\\"stylesheet\\\",\\\"href\\\":\\\"/_next/static/chunks/%5Broot-of-the-server%5D__0cbk-n2._.css\\\",\\\"precedence\\\":\\\"next_static/chunks/[root-of-the-server]__0cbk-n2._.css\\\",\\\"crossOrigin\\\":\\\"$undefined\\\",\\\"nonce\\\":\\\"$undefined\\\"},null,\\\"$c\\\",0],[\\\"$\\\",\\\"script\\\",\\\"script-0\\\",{\\\"src\\\":\\\"/_next/static/chunks/node_modules_0ti0-sa._.js\\\",\\\"async\\\":true,\\\"nonce\\\":\\\"$undefined\\\"},null,\\\"$d\\\",0],[\\\"$\\\",\\\"script\\\",\\\"script-1\\\",{\\\"src\\\":\\\"/_next/static/chunks/_16m5dxf._.js\\\",\\\"async\\\":true,\\\"nonce\\\":\\\"$undefined\\\"},null,\\\"$e\\\",0],[\\\"$\\\",\\\"script\\\",\\\"script-2\\\",{\\\"src\\\":\\\"/_next/static/chunks/app_layout_tsx_007e4b2._.js\\\",\\\"async\\\":true,\\\"nonce\\\":\\\"$undefined\\\"},null,\\\"$f\\\",0]],\\\"$10\\\"]},null,\\\"$a\\\",1]},null,\\\"$8\\\",0],{\\\"children\\\":[[\\\"$\\\",\\\"$b\\\",\\\"c\\\",{\\\"children\\\":[null,[\\\"$\\\",\\\"$L2a\\\",null,{\\\"parallelRouterKey\\\":\\\"children\\\",\\\"error\\\":\\\"$undefined\\\",\\\"errorStyles\\\":\\\"$undefined\\\",\\\"errorScripts\\\":\\\"$undefined\\\",\\\"template\\\":[\\\"$\\\",\\\"$L2c\\\",null,{},null,\\\"$41\\\",1],\\\"templateStyles\\\":\\\"$undefined\\\",\\\"templateScripts\\\":\\\"$undefined\\\",\\\"notFound\\\":\\\"$undefined\\\",\\\"forbidden\\\":\\\"$undefined\\\",\\\"unauthorized\\\":\\\"$undefined\\\",\\\"segmentViewBoundaries\\\":[\\\"$undefined\\\",\\\"$undefined\\\",\\\"$undefined\\\",\\\"$undefined\\\"]},null,\\\"$40\\\",1]]},null,\\\"$3f\\\",0],{\\\"children\\\":[[\\\"$\\\",\\\"$b\\\",\\\"c\\\",{\\\"children\\\":[null,[\\\"$\\\",\\\"$L2a\\\",null,{\\\"parallelRouterKey\\\":\\\"children\\\",\\\"error\\\":\\\"$undefined\\\",\\\"errorStyles\\\":\\\"$undefined\\\",\\\"errorScripts\\\":\\\"$undefined\\\",\\\"template\\\":[\\\"$\\\",\\\"$L2c\\\",null,{},null,\\\"$44\\\",1],\\\"templateStyles\\\":\\\"$undefined\\\",\\\"templateScripts\\\":\\\"$undefined\\\",\\\"notFound\\\":\\\"$undefined\\\",\\\"forbidden\\\":\\\"$undefined\\\",\\\"unauthorized\\\":\\\"$undefined\\\",\\\"segmentViewBoundaries\\\":[\\\"$undefined\\\",\\\"$undefined\\\",\\\"$undefined\\\",\\\"$undefined\\\"]},null,\\\"$43\\\",1]]},null,\\\"$42\\\",0],{\\\"children\\\":[[\\\"$\\\",\\\"$b\\\",\\\"c\\\",{\\\"children\\\":[[\\\"$\\\",\\\"$L9\\\",\\\"c-page\\\",{\\\"type\\\":\\\"page\\\",\\\"pagePath\\\":\\\"[entity]/new/page.tsx\\\",\\\"children\\\":[\\\"$\\\",\\\"$L48\\\",null,{\\\"Component\\\":\\\"$49\\\",\\\"serverProvidedParams\\\":{\\\"searchParams\\\":{},\\\"params\\\":{\\\"entity\\\":\\\"materials\\\"},\\\"promises\\\":null}},null,\\\"$47\\\",1]},null,\\\"$46\\\",1],[[\\\"$\\\",\\\"script\\\",\\\"script-0\\\",{\\\"src\\\":\\\"/_next/static/chunks/_0u99gvz._.js\\\",\\\"async\\\":true,\\\"nonce\\\":\\\"$undefined\\\"},null,\\\"$4a\\\",0],[\\\"$\\\",\\\"script\\\",\\\"script-1\\\",{\\\"src\\\":\\\"/_next/static/chunks/app_%5Bentity%5D_new_page_tsx_143_m3u._.js\\\",\\\"async\\\":true,\\\"nonce\\\":\\\"$undefined\\\"},null,\\\"$4b\\\",0]],\\\"$4c\\\"]},null,\\\"$45\\\",0],{},null,false,null]},null,false,\\\"$@55\\\"]},null,false,\\\"$@55\\\"]},null,false,null],[\\\"$\\\",\\\"$b\\\",\\\"h\\\",{\\\"children\\\":[\\\"$57\\\",\\\"$5c\\\",\\\"$65\\\",[\\\"$\\\",\\\"meta\\\",null,{\\\"name\\\":\\\"next-size-adjust\\\",\\\"content\\\":\\\"\\\"},null,\\\"$70\\\",1]]},null,\\\"$56\\\",0],false]],\\\"m\\\":\\\"$W71\\\",\\\"G\\\":[\\\"$72\\\",[\\\"$\\\",\\\"$L9\\\",\\\"ge-svn\\\",{\\\"type\\\":\\\"global-error\\\",\\\"pagePath\\\":\\\"__next_builtin__global-error.js\\\",\\\"children\\\":[[\\\"$\\\",\\\"link\\\",\\\"0\\\",{\\\"rel\\\":\\\"stylesheet\\\",\\\"href\\\":\\\"/_next/static/chunks/%5Broot-of-the-server%5D__0cbk-n2._.css\\\",\\\"precedence\\\":\\\"next_static/chunks/[root-of-the-server]__0cbk-n2._.css\\\",\\\"crossOrigin\\\":\\\"$undefined\\\",\\\"nonce\\\":\\\"$undefined\\\"},null,\\\"$74\\\",0]]},null,\\\"$73\\\",0]],\\\"S\\\":false,\\\"h\\\":null,\\\"s\\\":\\\"$undefined\\\",\\\"l\\\":\\\"$undefined\\\",\\\"p\\\":\\\"$undefined\\\",\\\"d\\\":\\\"$undefined\\\",\\\"b\\\":\\\"development\\\"}\\n\"])self.__next_f.push([1,\"75:[]\\n55:D\\\"$76\\\"\\n55:\\\"$W75\\\"\\n62:D\\\"$77\\\"\\n62:[[\\\"$\\\",\\\"meta\\\",\\\"0\\\",{\\\"charSet\\\":\\\"utf-8\\\"},\\\"$4d\\\",\\\"$78\\\",0],[\\\"$\\\",\\\"meta\\\",\\\"1\\\",{\\\"name\\\":\\\"viewport\\\",\\\"content\\\":\\\"width=device-width, initial-scale=1\\\"},\\\"$4d\\\",\\\"$79\\\",0]]\\n54:D\\\"$7a\\\"\\n54:null\\n6d:D\\\"$7b\\\"\\n6d:[[\\\"$\\\",\\\"title\\\",\\\"0\\\",{\\\"children\\\":\\\"Morphis Admin\\\"},\\\"$4d\\\",\\\"$7c\\\",0],[\\\"$\\\",\\\"meta\\\",\\\"1\\\",{\\\"name\\\":\\\"description\\\",\\\"content\\\":\\\"Generic GraphQL admin UI\\\"},\\\"$4d\\\",\\\"$7d\\\",0],[\\\"$\\\",\\\"link\\\",\\\"2\\\",{\\\"rel\\\":\\\"icon\\\",\\\"href\\\":\\\"/favicon.ico?favicon.2vob68tjqpejf.ico\\\",\\\"sizes\\\":\\\"256x256\\\",\\\"type\\\":\\\"image/x-icon\\\"},\\\"$4d\\\",\\\"$7e\\\",0],[\\\"$\\\",\\\"link\\\",\\\"3\\\",{\\\"rel\\\":\\\"icon\\\",\\\"href\\\":\\\"/icon.svg?icon.1u9c9bmajc_ou.svg\\\",\\\"sizes\\\":\\\"any\\\",\\\"type\\\":\\\"image/svg+xml\\\"},\\\"$4d\\\",\\\"$7f\\\",0],[\\\"$\\\",\\\"$L81\\\",\\\"4\\\",{},\\\"$4d\\\",\\\"$80\\\",0]]\\n\"])[Network] Internal Server Error"
```

# Page snapshot

```yaml
- generic [active] [ref=e1]:
  - banner [ref=e2]:
    - generic [ref=e3]:
      - link "Morphis Admin" [ref=e4] [cursor=pointer]:
        - /url: /
        - img [ref=e5]
        - generic [ref=e9]: Morphis Admin
      - generic [ref=e10]:
        - navigation [ref=e11]:
          - link "Colorways" [ref=e12] [cursor=pointer]:
            - /url: /colorways
          - link "Feature Attributes" [ref=e13] [cursor=pointer]:
            - /url: /feature_attributes
          - link "Material Features" [ref=e14] [cursor=pointer]:
            - /url: /material_features
          - link "Materials" [ref=e15] [cursor=pointer]:
            - /url: /materials
          - link "protected data" [ref=e16] [cursor=pointer]:
            - /url: /protected_data
          - link "Sizes" [ref=e17] [cursor=pointer]:
            - /url: /sizes
          - link "user permissions" [ref=e18] [cursor=pointer]:
            - /url: /user_permissions
        - button "EN" [ref=e19]
        - button "中文" [ref=e20]
        - button "Switch to dark mode" [ref=e21]:
          - img [ref=e22]
  - main [ref=e24]:
    - generic [ref=e25]:
      - navigation [ref=e26]:
        - link "Entities" [ref=e28] [cursor=pointer]:
          - /url: /
        - generic [ref=e29]:
          - img [ref=e30]
          - link "Materials" [ref=e32] [cursor=pointer]:
            - /url: /materials
        - generic [ref=e33]:
          - img [ref=e34]
          - generic [ref=e36]: New
      - generic [ref=e37]:
        - heading "New Materials" [level=1] [ref=e38]
        - paragraph [ref=e39]: Create a new record
        - generic [ref=e41]:
          - generic [ref=e42]:
            - generic [ref=e43]: Mat. No.*
            - textbox "Mat. No.*" [ref=e44]: E2E_1780826060790
          - generic [ref=e45]:
            - generic [ref=e46]: Name*
            - textbox "Name*" [ref=e47]: E2E Test Material
          - generic [ref=e48]:
            - generic [ref=e49]: Status*
            - combobox "Status*" [ref=e50]:
              - option "-- Select --"
              - option "Active" [selected]
              - option "Discontinued"
          - generic [ref=e51]: "[Network] Internal Server Error"
          - button "Create" [ref=e52]
  - button "Open Next.js Dev Tools" [ref=e58] [cursor=pointer]:
    - img [ref=e59]
  - alert [ref=e62]
  - generic [ref=e64]:
    - img [ref=e65]
    - generic [ref=e67]: "[Network] Internal Server Error"
```

# Test source

```ts
  1   | import { test, expect } from "@playwright/test";
  2   | 
  3   | test.describe("Full CRUD flow", () => {
  4   |   test("create, view, edit, and delete a material", async ({ page }) => {
  5   |     const logs: string[] = [];
  6   |     page.on("console", (msg) => {
  7   |       if (msg.type() === "error") logs.push(`[${msg.type()}] ${msg.text()}`);
  8   |     });
  9   |     page.on("pageerror", (err) => logs.push(`[PAGE ERROR] ${err.message}`));
  10  | 
  11  |     const testMatNo = `E2E_${Date.now()}`;
  12  | 
  13  |     // 1. Navigate to /materials/new
  14  |     await page.goto("http://localhost:3000/materials/new", {
  15  |       waitUntil: "networkidle",
  16  |     });
  17  |     await page.waitForTimeout(1500);
  18  |     console.log("=== Navigate to new ===");
  19  |     console.log("URL:", page.url());
  20  |     console.log("Errors:", logs);
  21  |     logs.length = 0;
  22  | 
  23  |     // Check the page loaded
  24  |     const body = await page.textContent("body");
  25  |     expect(body).toContain("New Materials");
  26  | 
  27  |     // 2. Fill in the create form
  28  |     // Find inputs by their labels
  29  |     await page.fill('input[name="mat_no"]', testMatNo);
  30  |     await page.fill('input[name="name"]', "E2E Test Material");
  31  |     await page.selectOption('select[name="status"]', "active");
  32  | 
  33  |     // 3. Submit
  34  |     await page.click('button[type="submit"]');
  35  |     await page.waitForTimeout(2000);
  36  | 
  37  |     // Check for errors
  38  |     console.log("=== After create submit ===");
  39  |     console.log("URL:", page.url());
  40  |     console.log("Errors:", logs);
  41  |     logs.length = 0;
  42  | 
  43  |     // Should redirect to /materials list
  44  |     expect(page.url()).toContain("/materials");
  45  | 
  46  |     // 4. Verify the new material appears in the list
  47  |     const listBody = await page.textContent("body");
> 48  |     expect(listBody).toContain(testMatNo);
      |                      ^ Error: expect(received).toContain(expected) // indexOf
  49  |     expect(listBody).toContain("E2E Test Material");
  50  | 
  51  |     // 5. Click Edit on the new material
  52  |     await page.goto(
  53  |       `http://localhost:3000/materials/${encodeURIComponent(testMatNo)}`,
  54  |       { waitUntil: "networkidle" }
  55  |     );
  56  |     await page.waitForTimeout(1500);
  57  | 
  58  |     console.log("=== Navigate to edit ===");
  59  |     console.log("URL:", page.url());
  60  |     console.log("Errors:", logs);
  61  |     logs.length = 0;
  62  | 
  63  |     const editBody = await page.textContent("body");
  64  |     expect(editBody).toContain(testMatNo);
  65  | 
  66  |     // 6. Update the name
  67  |     await page.fill('input[name="name"]', "E2E Updated");
  68  |     await page.click('button[type="submit"]');
  69  |     await page.waitForTimeout(2000);
  70  | 
  71  |     console.log("=== After update submit ===");
  72  |     console.log("URL:", page.url());
  73  |     console.log("Errors:", logs);
  74  |     logs.length = 0;
  75  | 
  76  |     expect(page.url()).toContain("/materials");
  77  | 
  78  |     // 7. Verify the update in the list
  79  |     const updatedBody = await page.textContent("body");
  80  |     expect(updatedBody).toContain("E2E Updated");
  81  | 
  82  |     // 8. Delete the test material
  83  |     // Click Delete button for our test row
  84  |     await page.goto(`http://localhost:3000/materials`, {
  85  |       waitUntil: "networkidle",
  86  |     });
  87  |     await page.waitForTimeout(1500);
  88  | 
  89  |     // page.on('dialog') doesn't work great with confirm, let's use the API directly
  90  |     const deleteRes = await page.evaluate(async (matNo) => {
  91  |       const res = await fetch("http://localhost:4000/graphql", {
  92  |         method: "POST",
  93  |         headers: { "Content-Type": "application/json" },
  94  |         body: JSON.stringify({
  95  |           query: `mutation { deleteMaterials(id: "${matNo}") { mat_no } }`,
  96  |         }),
  97  |       });
  98  |       return res.json();
  99  |     }, testMatNo);
  100 | 
  101 |     console.log("=== Delete result ===");
  102 |     console.log(JSON.stringify(deleteRes));
  103 | 
  104 |     await page.reload({ waitUntil: "networkidle" });
  105 |     await page.waitForTimeout(1000);
  106 | 
  107 |     const finalBody = await page.textContent("body");
  108 |     expect(finalBody).not.toContain("E2E Updated");
  109 |   });
  110 | });
  111 | 
```