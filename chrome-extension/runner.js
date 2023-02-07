var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
  function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
  return new (P || (P = Promise))(function (resolve, reject) {
      function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
      function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
      function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
      step((generator = generator.apply(thisArg, _arguments || [])).next());
  });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
  var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
  return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
  function verb(n) { return function (v) { return step([n, v]); }; }
  function step(op) {
      if (f) throw new TypeError("Generator is already executing.");
      while (g && (g = 0, op[0] && (_ = 0)), _) try {
          if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
          if (y = 0, t) op = [op[0] & 2, t.value];
          switch (op[0]) {
              case 0: case 1: t = op; break;
              case 4: _.label++; return { value: op[1], done: false };
              case 5: _.label++; y = op[1]; op = [0]; continue;
              case 7: op = _.ops.pop(); _.trys.pop(); continue;
              default:
                  if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                  if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                  if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                  if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                  if (t[2]) _.ops.pop();
                  _.trys.pop(); continue;
          }
          op = body.call(thisArg, _);
      } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
      if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
  }
};
(function (exports) {
  var a11y = (exports.__a11y = {
      run: runA11y,
      runners: {},
  });
  var issueCodeMap = {
      unknown: 0,
      error: 1,
      warning: 2,
      notice: 3,
  };
  var rootElement = null;
  var hiddenElements = null;
  function shapeIssue(issue) {
      var context = "";
      var selector = "";
      if (issue.element) {
          context = getElementContext(issue.element);
          selector = getElementSelector(issue.element);
      }
      return {
          context: context,
          selector: selector,
          code: issue.code,
          type: issue.type,
          typeCode: issueCodeMap[issue.type] || 0,
          message: issue.message,
          runner: issue.runner || "a11y",
          runnerExtras: issue.runnerExtras || {},
          recurrence: issue.recurrence || 0,
      };
  }
  function runA11y(options) {
      return __awaiter(this, void 0, void 0, function () {
          function isIssueNotIgnored(issue) {
              return !options.ignore.some(function (element) {
                  return element === issue.type || element === issue.code.toLowerCase();
              });
          }
          function isElementInTestArea(element) {
              if (!rootElement) {
                  rootElement = window.document.querySelector(options.rootElement);
              }
              return rootElement ? rootElement.contains(element) : true;
          }
          function isElementOutsideHiddenArea(element) {
              if (!hiddenElements) {
                  hiddenElements = window.document.querySelectorAll(options.hideElements);
              }
              var found = true;
              if (hiddenElements && hiddenElements.length) {
                  found = false;
                  for (var i = 0; i < hiddenElements.length; i++) {
                      if (hiddenElements[i].contains(element)) {
                          found = true;
                          break;
                      }
                  }
              }
              return found;
          }
          function validateIssue(issue) {
              return ((options.rootElement && !isElementInTestArea(issue.element)) ||
                  (options.hideElements && !isElementOutsideHiddenArea(issue.element)) ||
                  !isIssueNotIgnored(issue));
          }
          function processIssues(issues, meta, missingAltIndexs) {
              var acc = new Array((issues && issues.length) || 0);
              var ic = 0;
              var _loop_1 = function (i) {
                  if (validateIssue(issues[i])) {
                      return "continue";
                  }
                  if (issues[i].type === "error") {
                      if (issues[i].code === "WCAG2AA.Principle1.Guideline1_1.1_1_1.H37") {
                          missingAltIndexs.push(ic);
                      }
                      acc[ic] = shapeIssue(issues[i]);
                      ic++;
                      meta.errorCount++;
                      meta.accessScore -= 2;
                  }
                  else {
                      queueMicrotask(function () {
                          if (issues[i].type === "warning") {
                              meta.warningCount++;
                          }
                          if (issues[i].type === "notice") {
                              meta.noticeCount++;
                          }
                          acc[ic] = shapeIssue(issues[i]);
                          ic++;
                      });
                  }
              };
              for (var i = 0; i < acc.length; i++) {
                  _loop_1(i);
              }
              acc.length = ic;
              return acc;
          }
          function processIssuesMulti(issues, acc, ic, meta, missingAltIndexs) {
              var _loop_2 = function (i) {
                  if (validateIssue(issues[i])) {
                      return "continue";
                  }
                  if (issues[i].type === "error") {
                      if (issues[i].code === "WCAG2AA.Principle1.Guideline1_1.1_1_1.H37") {
                          missingAltIndexs.push(ic);
                      }
                      acc[ic] = shapeIssue(issues[i]);
                      ic++;
                      meta.errorCount++;
                      meta.accessScore -= 2;
                  }
                  else {
                      queueMicrotask(function () {
                          if (issues[i].type === "warning") {
                              meta.warningCount++;
                          }
                          if (issues[i].type === "notice") {
                              meta.noticeCount++;
                          }
                          acc[ic] = shapeIssue(issues[i]);
                          ic++;
                      });
                  }
              };
              for (var i = 0; i < issues.length; i++) {
                  _loop_2(i);
              }
              return ic;
          }
          var runnerIssues, meta, missingAltIndexs, issues, ic, _i, runnerIssues_1, is;
          return __generator(this, function (_a) {
              switch (_a.label) {
                  case 0: return [4, Promise.all(options.runners.map(function (runner) {
                          return a11y.runners[runner](options, a11y).catch(function (e) {
                              console.error(e);
                              return [];
                          });
                      }))];
                  case 1:
                      runnerIssues = _a.sent();
                      meta = {
                          errorCount: 0,
                          warningCount: 0,
                          noticeCount: 0,
                          accessScore: 100,
                          possibleIssuesFixedByCdn: 0,
                      };
                      missingAltIndexs = [];
                      if (runnerIssues.length === 1) {
                          return [2, {
                                  documentTitle: window.document.title,
                                  pageUrl: window.location.href,
                                  issues: processIssues(runnerIssues[0], meta, missingAltIndexs),
                                  meta: meta,
                                  automateable: {
                                      missingAltIndexs: missingAltIndexs,
                                  },
                              }];
                      }
                      issues = new Array(runnerIssues.reduce(function (ac, cv) { return ac + cv.length; }, 0));
                      ic = 0;
                      for (_i = 0, runnerIssues_1 = runnerIssues; _i < runnerIssues_1.length; _i++) {
                          is = runnerIssues_1[_i];
                          ic = processIssuesMulti(is, issues, ic, meta, missingAltIndexs);
                      }
                      issues.length = ic;
                      return [2, {
                              documentTitle: window.document.title,
                              pageUrl: window.location.href,
                              issues: issues,
                              meta: meta,
                              automateable: {
                                  missingAltIndexs: missingAltIndexs,
                              },
                          }];
              }
          });
      });
  }
  function getElementContext(element) {
      var outerHTML = element.outerHTML;
      if (!outerHTML) {
          return "";
      }
      if (element.innerHTML.length > 31) {
          outerHTML = outerHTML.replace(element.innerHTML, "".concat(element.innerHTML.substring(0, 31), "..."));
      }
      if (outerHTML.length > 251) {
          outerHTML = "".concat(outerHTML.substring(0, 250), "...");
      }
      return outerHTML;
  }
  function isElementNode(element) {
      return element.nodeType === window.Node.ELEMENT_NODE;
  }
  function getElementSelector(element, selectorParts) {
      if (selectorParts === void 0) { selectorParts = []; }
      if (isElementNode(element)) {
          selectorParts.unshift(buildElementIdentifier(element));
          if (!element.id && element.parentNode) {
              return getElementSelector(element.parentNode, selectorParts);
          }
      }
      return selectorParts.join(" > ");
  }
  function getSiblings(element) {
      var dupSibling = 0;
      var siblingIndex = 0;
      for (var _i = 0, _a = element.parentNode.childNodes; _i < _a.length; _i++) {
          var node = _a[_i];
          if (isElementNode(node)) {
              siblingIndex++;
          }
          if (node.tagName === element.tagName) {
              dupSibling += 1;
              if (dupSibling === 2) {
                  break;
              }
          }
      }
      return {
          siblingIndex: siblingIndex,
          onlySibling: dupSibling <= 1,
      };
  }
  function buildElementIdentifier(element) {
      if (element.id) {
          return "".concat(element.id[0] !== "#" ? "#" : "").concat(element.id);
      }
      var identifier = element.tagName.toLowerCase();
      if (!element.parentNode) {
          return identifier;
      }
      var _a = getSiblings(element), onlySibling = _a.onlySibling, siblingIndex = _a.siblingIndex;
      if (!onlySibling) {
          identifier += ":nth-child(".concat(siblingIndex + 1, ")");
      }
      return identifier;
  }
})(this);
//# sourceMappingURL=runner.js.map