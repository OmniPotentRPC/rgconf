;; Copyright 2023--present rgconf developers
;;
;; Publish docs/orgmode into docs/source as reStructuredText for Sphinx.
;; When ox-rst is on the load path it is required so the stack's Org
;; exporter stays available. The file writer below is what places each
;; page under docs/source, including when ox-rst is not installed.

(require 'ox-rst nil t)

(defvar rgconf-org-root (expand-file-name "orgmode" (file-name-directory load-file-name)))
(defvar rgconf-rst-root (expand-file-name "source" (file-name-directory load-file-name)))

(defun rgconf-verbatim (text)
  "Turn Org =verbatim= into RST literals."
  (replace-regexp-in-string "=\\([^=\n]+\\)=" "``\\1``" text))

(defun rgconf-underline (title level)
  "Underline TITLE for an Org heading LEVEL."
  (let ((char (aref "=-~^" (1- (min level 4)))))
    (concat title "\n" (make-string (length title) char) "\n")))

(defun rgconf-plain-convert (org rst)
  "Convert ORG to RST at RST."
  (with-temp-buffer
    (insert-file-contents org)
    (goto-char (point-min))
    (let ((chunks '())
          (in-src nil))
      (while (not (eobp))
        (let ((line (rgconf-verbatim (buffer-substring-no-properties (line-beginning-position) (line-end-position)))))
          (cond
           ((and in-src (string-match "^#\\+END_SRC" line))
            (setq in-src nil)
            (push "\n" chunks))
           (in-src
            (push (concat "   " line "\n") chunks))
           ((string-match "^#\\+BEGIN_SRC +\\([a-zA-Z0-9]+\\)" line)
            (setq in-src t)
            (push (format "\n.. code-block:: %s\n\n" (match-string 1 line)) chunks))
           ((string-match "^#\\+TITLE: +\\(.*\\)" line)
            (push (concat (rgconf-underline (match-string 1 line) 1) "\n") chunks))
           ((string-match "^\\(\\*+\\) +\\(.*\\)" line)
            (push (concat "\n" (rgconf-underline (match-string 2 line) (length (match-string 1 line))) "\n") chunks))
           ((string-match "^- +\\(.*\\)" line)
            (push (format "* %s\n" (match-string 1 line)) chunks))
           ((string-match "^\\s-*# " line) nil)
           (t (push (concat line "\n") chunks))))
        (forward-line 1))
      (when in-src (error "Unclosed source block in %s" org))
      (make-directory (file-name-directory rst) t)
      (with-temp-file rst
        (dolist (chunk (nreverse chunks))
          (insert chunk))))))

(defun rgconf-export-all ()
  "Export every Org file under docs/orgmode."
  (dolist (org (directory-files-recursively rgconf-org-root "\\.org$"))
    (let* ((rel (file-relative-name org rgconf-org-root))
           (rst (expand-file-name (concat (file-name-sans-extension rel) ".rst") rgconf-rst-root)))
      (rgconf-plain-convert org rst)
      (message "exported %s" rel))))

(rgconf-export-all)
